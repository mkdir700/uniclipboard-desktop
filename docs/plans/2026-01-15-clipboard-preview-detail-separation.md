# Clipboard Preview/Detail Separation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Separate clipboard entry preview from full content to optimize list loading performance - entries list returns truncated preview (500 chars), detail fetched on-demand when user expands.

**Architecture:**

- **Storage:** Materializer truncates large text to 500 chars in `inline_data`, non-text content (images) skips inline when >16KB
- **Backend:** Modify `get_clipboard_entries` to use inline_data as preview, add new `get_clipboard_entry_detail` UseCase for full content
- **Frontend:** Replace `is_truncated` with `has_detail` field, fetch detail only on user expand action

**Tech Stack:** Rust (Tauri 2, Hexagonal Architecture), React (TypeScript), SQLite, Diesel ORM

---

## Task 1: Add Text Detection Helper to Materializer

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/materializer.rs`

**Step 1: Write failing test**

```rust
// src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::clipboard::mime_type::MimeType;

    #[test]
    fn test_is_text_mime_type_with_text_plain() {
        assert!(is_text_mime_type(&Some(MimeType::new("text/plain"))));
    }

    #[test]
    fn test_is_text_mime_type_with_json() {
        assert!(is_text_mime_type(&Some(MimeType::new("application/json"))));
    }

    #[test]
    fn test_is_text_mime_type_with_image() {
        assert!(!is_text_mime_type(&Some(MimeType::new("image/png"))));
    }

    #[test]
    fn test_is_text_mime_type_with_none() {
        assert!(!is_text_mime_type(&None));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-infra test_is_text_mime_type -- --nocapture`
Expected: COMPILER ERROR "cannot find function `is_text_mime_type`"

**Step 3: Write minimal implementation**

```rust
// src-tauri/crates/uc-infra/src/clipboard/materializer.rs

const PREVIEW_LENGTH_CHARS: usize = 500;

fn is_text_mime_type(mime_type: &Option<MimeType>) -> bool {
    match mime_type {
        None => false,
        Some(mt) => {
            let mt_str = mt.as_str();
            mt_str.starts_with("text/") ||
            mt_str == "text/plain" ||
            mt_str.contains("json") ||
            mt_str.contains("xml") ||
            mt_str.contains("javascript") ||
            mt_str.contains("html") ||
            mt_str.contains("css")
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-infra test_is_text_mime_type -- --nocapture`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/materializer.rs
git add src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs
git commit -m "feat: add text MIME type detection helper

Add is_text_mime_type() helper to identify text-based content types.
This is used to determine whether to generate inline preview for large content.

Related: #clipboard-preview-detail"
```

---

## Task 2: Add Preview Truncation Helper

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/materializer.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_truncate_to_preview_ascii() {
    let input = b"hello".repeat(1000); // 5000 bytes
    let result = truncate_to_preview(&input);
    assert_eq!(result.len(), 500); // 500 chars
    assert_eq!(String::from_utf8_lossy(&result), "h".repeat(500));
}

#[test]
fn test_truncate_to_preview_utf8() {
    // Chinese characters are 3 bytes each in UTF-8
    let input = "你好".repeat(500).as_bytes().to_vec(); // 3000 bytes
    let result = truncate_to_preview(&input);
    assert_eq!(String::from_utf8_lossy(&result), "你好".repeat(250)); // 500 chars
}

#[test]
fn test_truncate_to_preview_shorter_than_limit() {
    let input = b"short";
    let result = truncate_to_preview(input);
    assert_eq!(result, b"short");
}

#[test]
fn test_truncate_to_preview_invalid_utf8() {
    let input = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8
    let result = truncate_to_preview(&input);
    // Fallback to byte truncation
    assert_eq!(result.len(), 3);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-infra test_truncate_to_preview -- --nocapture`
Expected: COMPILER ERROR "cannot find function `truncate_to_preview`"

**Step 3: Write minimal implementation**

```rust
// src-tauri/crates/uc-infra/src/clipboard/materializer.rs

fn truncate_to_preview(bytes: &[u8]) -> Vec<u8> {
    // UTF-8 safe truncation to first N characters
    std::str::from_utf8(bytes)
        .map(|text| {
            text.chars()
                .take(PREVIEW_LENGTH_CHARS)
                .collect::<String>()
                .into_bytes()
        })
        .unwrap_or_else(|_| {
            // Fallback for invalid UTF-8: truncate bytes
            bytes[..bytes.len().min(PREVIEW_LENGTH_CHARS)].to_vec()
        })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-infra test_truncate_to_preview -- --nocapture`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/materializer.rs
git add src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs
git commit -m "feat: add UTF-8 safe preview truncation helper

Add truncate_to_preview() to truncate text to 500 characters.
Handles UTF-8 multi-byte characters correctly with fallback for invalid UTF-8.

Related: #clipboard-preview-detail"
```

---

## Task 3: Modify Materializer to Store Preview for Large Text

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/materializer.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs`

**Step 1: Write failing test**

```rust
#[tokio::test]
async fn test_materialize_large_text_creates_inline_preview() {
    let db = setup_test_db().await;
    let blob_repo = TestBlobRepository::new();

    // Create text larger than inline threshold (16KB)
    let large_text = "x".repeat(20000);
    let observed = ObservedClipboardRepresentation {
        format_id: FormatId::new("public.utf8-plain-text"),
        mime_type: Some(MimeType::new("text/plain")),
        bytes: large_text.as_bytes().to_vec(),
    };

    let materializer = ClipboardMaterializer::new(db.clone(), blob_repo.clone());
    let result = materializer.materialize(observed).await.unwrap();

    // Should have both inline preview AND blob
    assert!(result.inline_data.is_some(), "Should have inline preview");
    assert!(result.blob_id.is_some(), "Should have blob for full content");

    // Inline should be truncated to 500 chars
    let inline_text = String::from_utf8(result.inline_data.unwrap()).unwrap();
    assert_eq!(inline_text.len(), 500);
    assert_eq!(inline_text, "x".repeat(500));

    // Blob should contain full content
    let blob = blob_repo.get(&result.blob_id.unwrap()).await.unwrap();
    assert_eq!(blob.content.len(), 20000);
}

#[tokio::test]
async fn test_materialize_large_image_no_inline() {
    let db = setup_test_db().await;
    let blob_repo = TestBlobRepository::new();

    // Create image data larger than inline threshold
    let large_image = vec![0u8; 20000];
    let observed = ObservedClipboardRepresentation {
        format_id: FormatId::new("public.png"),
        mime_type: Some(MimeType::new("image/png")),
        bytes: large_image.clone(),
    };

    let materializer = ClipboardMaterializer::new(db.clone(), blob_repo.clone());
    let result = materializer.materialize(observed).await.unwrap();

    // Should have NO inline data, only blob
    assert!(result.inline_data.is_none(), "Large images should not have inline data");
    assert!(result.blob_id.is_some(), "Should have blob");

    // Blob should contain full content
    let blob = blob_repo.get(&result.blob_id.unwrap()).await.unwrap();
    assert_eq!(blob.content, large_image);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p uc-infra test_materialize_large -- --nocapture`
Expected: FAIL (current implementation sets inline_data to None for large content)

**Step 3: Modify implementation**

```rust
// src-tauri/crates/uc-infra/src/clipboard/materializer.rs

// Find the inline_data assignment in the materialize() method
// Replace the existing logic with:

use uc_core::clipboard::config::ClipboardStorageConfig;

// Inside the materialize() method, after calculating size_bytes:
let config = ClipboardStorageConfig::defaults();
let inline_threshold_bytes = config.inline_threshold_bytes;

// Decision: inline or blob, with preview for large text
let inline_data = if size_bytes <= inline_threshold_bytes {
    // Small content: store full data inline
    Some(observed.bytes.clone())
} else {
    // Large content: decide based on type
    if is_text_mime_type(&observed.mime_type) {
        // Text type: generate truncated preview
        Some(truncate_to_preview(&observed.bytes))
    } else {
        // Non-text (images, etc.): no inline, blob only
        None
    }
};

// blob_id logic remains the same
let blob_id = if size_bytes <= inline_threshold_bytes {
    None
} else {
    Some(blob_id) // existing blob creation logic
};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p uc-infra test_materialize_large -- --nocapture`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/materializer.rs
git add src-tauri/crates/uc-infra/src/clipboard/materializer_test.rs
git commit -m "feat: store preview inline for large text content

Materializer now generates 500-char preview for text content >16KB.
Large images skip inline storage (blob only).
This enables efficient list queries without reading blobs.

Changes:
- Text >16KB: inline preview (500 chars) + blob full content
- Images >16KB: blob only (no inline)
- Content <=16KB: unchanged (inline only)

Related: #clipboard-preview-detail"
```

---

## Task 4: Update ClipboardEntryProjection Model

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/models/mod.rs`

**Step 1: Modify model structure**

```rust
// src-tauri/crates/uc-tauri/src/models/mod.rs

/// Clipboard entry projection for frontend API.
/// 前端 API 的剪贴板条目投影。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryProjection {
    /// Unique identifier for the entry
    pub id: String,
    /// Preview content (truncated for large text, placeholder for images)
    pub preview: String,
    /// Whether full detail is available (has blob or is expandable)
    pub has_detail: bool,
    /// Total size in bytes
    pub size_bytes: i64,
    /// Timestamp when captured (Unix timestamp)
    pub captured_at: i64,
    /// Content type description
    pub content_type: String,
    /// Whether the content is encrypted
    pub is_encrypted: bool,
    /// Whether the entry is favorited
    pub is_favorited: bool,
    /// Timestamp when last updated
    pub updated_at: i64,
    /// Timestamp of last access/use
    pub active_time: i64,
}

// Add new model for detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryDetail {
    /// Unique identifier for the entry
    pub id: String,
    /// Full content
    pub content: String,
    /// Total size in bytes
    pub size_bytes: i64,
    /// Content type description
    pub content_type: String,
    /// Whether the entry is favorited
    pub is_favorited: bool,
    /// Timestamp when last updated
    pub updated_at: i64,
    /// Timestamp of last access/use
    pub active_time: i64,
}
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-tauri`
Expected: May have compilation errors in commands that use this model (fix in next task)

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/models/mod.rs
git commit -m "feat: update ClipboardEntryProjection for preview/detail split

Changes:
- Rename 'preview' field (was misnamed, now accurate)
- Add 'has_detail' boolean to indicate expandability
- Add ClipboardEntryDetail model for full content API

Related: #clipboard-preview-detail"
```

---

## Task 5: Modify get_clipboard_entries Command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`

**Step 1: Update command implementation**

```rust
// src-tauri/crates/uc-tauri/src/commands/clipboard.rs

/// Get clipboard history entries (preview only)
/// 获取剪贴板历史条目（仅预览）
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let resolved_limit = limit.unwrap_or(50);
    let device_id = runtime.deps.device_identity.current_device_id();

    log::info!(
        "Getting clipboard entries: device_id={}, limit={}",
        device_id,
        resolved_limit
    );

    let uc = runtime.usecases().list_clipboard_entries();
    let entries = uc.execute(resolved_limit, 0).await.map_err(|e| {
        log::error!("Failed to get clipboard entries: {}", e);
        e.to_string()
    })?;

    let mut projections = Vec::with_capacity(entries.len());

    for entry in entries {
        let captured_at = entry.created_at_ms;

        // Get preview from inline_data (already truncated if large)
        let (preview, has_detail) = if let Ok(Some(selection)) = runtime
            .deps
            .selection_repo
            .get_selection(&entry.entry_id)
            .await
        {
            if let Ok(rep) = runtime
                .deps
                .clipboard_event_reader
                .get_representation(
                    &entry.event_id,
                    selection.selection.preview_rep_id.as_ref(),
                )
                .await
            {
                let preview_text = if let Some(data) = rep.inline_data {
                    String::from_utf8_lossy(&data).trim().to_string()
                } else {
                    // Large image with no inline: show placeholder
                    format!("Image ({} bytes)", rep.size_bytes)
                };

                // has_detail = blob exists (content was truncated or is blob-only)
                let has_detail = rep.blob_id.is_some();

                (preview_text, has_detail)
            } else {
                (
                    entry.title.unwrap_or_else(|| {
                        format!("Entry ({} bytes)", entry.total_size)
                    }),
                    false
                )
            }
        } else {
            (
                entry.title.unwrap_or_else(|| {
                    format!("Entry ({} bytes)", entry.total_size)
                }),
                false
            )
        };

        projections.push(ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview,
            has_detail,
            size_bytes: entry.total_size,
            captured_at,
            content_type: "clipboard".to_string(),
            is_encrypted: false,
            is_favorited: false,
            updated_at: captured_at,
            active_time: captured_at,
        });
    }

    log::info!("Retrieved {} clipboard entries", projections.len());
    Ok(projections)
}
```

**Step 2: Run cargo check**

Run: `cargo check -p uc-tauri`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git commit -m "feat: return preview in get_clipboard_entries

Use inline_data directly as preview (already truncated in Materializer).
Set has_detail=true when blob exists.

Related: #clipboard-preview-detail"
```

---

## Task 6: Create GetEntryDetail UseCase

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/clipboard/get_entry_detail.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Create UseCase file**

```rust
// src-tauri/crates/uc-app/src/usecases/clipboard/get_entry_detail.rs

use uc_core::clipboard::ports::ClipboardEntryRepository;
use uc_core::ids::EntryId;
use uc_core::clipboard::snapshot::{PersistedClipboardEntry, Selection};
use uc_tauri::models::ClipboardEntryDetail;

pub struct GetEntryDetailUseCase<R: ClipboardEntryRepository> {
    repo: R,
}

impl<R: ClipboardEntryRepository> GetEntryDetailUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        entry_id: &EntryId,
    ) -> Result<ClipboardEntryDetail, Box<dyn std::error::Error>> {
        // Get entry
        let entry = self.repo.get_entry(entry_id).await?
            .ok_or("Entry not found")?;

        // Get selection
        let selection = self.repo.get_selection(entry_id).await?
            .ok_or("Selection not found")?;

        // Get preview representation
        let preview_rep = self.repo.get_representation(
            &entry.event_id,
            selection.preview_rep_id.as_ref()
        ).await?
            .ok_or("Preview representation not found")?;

        // Determine if we need to read from blob
        let full_content = if let Some(blob_id) = preview_rep.blob_id {
            // Read from blob
            let blob = self.repo.get_blob(&blob_id).await?
                .ok_or("Blob not found")?;
            String::from_utf8_lossy(&blob.content).to_string()
        } else {
            // Use inline data
            String::from_utf8_lossy(
                preview_rep.inline_data.as_ref().ok_or("No inline data")?
            ).to_string()
        };

        Ok(ClipboardEntryDetail {
            id: entry.entry_id.to_string(),
            content: full_content,
            size_bytes: entry.total_size,
            content_type: "clipboard".to_string(),
            is_favorited: false, // TODO: implement favorites
            updated_at: entry.created_at_ms,
            active_time: entry.created_at_ms,
        })
    }
}
```

**Step 2: Register UseCase in mod.rs**

```rust
// src-tauri/crates/uc-app/src/usecases/mod.rs

pub mod clipboard;
// ... existing imports ...

use clipboard::get_entry_detail::GetEntryDetailUseCase;

// In the UseCases struct:
pub struct UseCases<C, R, S, E, CR, CSR> {
    // ... existing fields ...
    pub get_entry_detail: Arc<GetEntryDetailUseCase<R>>,
}

impl<C, R, S, E, CR, CSR> UseCases<C, R, S, E, CR, CSR>
where
    R: ClipboardEntryRepository + 'static,
{
    pub fn new(/* ... existing args ... */) -> Self {
        // ... existing code ...
        let get_entry_detail = Arc::new(GetEntryDetailUseCase::new(list_entries_repo.clone()));

        Self {
            // ... existing fields ...
            get_entry_detail,
        }
    }
}
```

**Step 3: Add accessor in runtime.rs**

```rust
// src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs

impl AppRuntime {
    pub fn usecases(&self) -> &UseCases {
        &self.use_cases
    }

    // Add accessor
    pub fn get_entry_detail(&self) -> GetEntryDetailUseCaseImpl {
        GetEntryDetailUseCaseImpl::new(self.clone())
    }
}
```

**Step 4: Run cargo check**

Run: `cargo check -p uc-app -p uc-tauri`
Expected: May need to adjust trait bounds and imports

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat: add GetEntryDetailUseCase

New UseCase to fetch full clipboard entry content.
Reads from blob if exists, otherwise uses inline data.

Related: #clipboard-preview-detail"
```

---

## Task 7: Add get_clipboard_entry_detail Command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add command implementation**

```rust
// src-tauri/crates/uc-tauri/src/commands/clipboard.rs

use crate::models::ClipboardEntryDetail;
use uc_core::ids::EntryId;

/// Get full clipboard entry detail
/// 获取剪切板条目详情
#[tauri::command]
pub async fn get_clipboard_entry_detail(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<ClipboardEntryDetail, String> {
    log::info!("Getting clipboard entry detail: entry_id={}", entry_id);

    let parsed_id = EntryId::from(entry_id.clone());
    let use_case = runtime.usecases().get_entry_detail();

    use_case.execute(&parsed_id).await.map_err(|e| {
        log::error!("Failed to get entry detail {}: {}", entry_id, e);
        e.to_string()
    })
}
```

**Step 2: Register command in main.rs**

```rust
// src-tauri/src/main.rs

.invoke_handler(|_app| {
    // ... existing commands ...
    uc_tauri::commands::clipboard::get_clipboard_entries,
    uc_tauri::commands::clipboard::get_clipboard_entry_detail,  // NEW
    uc_tauri::commands::clipboard::delete_clipboard_entry,
    // ...
})
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/clipboard.rs
git add src-tauri/src/main.rs
git commit -m "feat: add get_clipboard_entry_detail command

New Tauri command to fetch full clipboard entry content.
Frontend calls this when user expands an entry.

Related: #clipboard-preview-detail"
```

---

## Task 8: Update Frontend Types

**Files:**

- Modify: `src/api/clipboardItems.ts`

**Step 1: Update type definitions**

```typescript
// src/api/clipboardItems.ts

// Backend projection type
interface ClipboardEntryProjection {
  id: string
  preview: string // Preview content (may be truncated)
  has_detail: boolean // Whether full detail is available
  size_bytes: number
  captured_at: number
  content_type: string
  is_encrypted: boolean
  is_favorited: boolean
  updated_at: number
  active_time: number
}

// Detail response type (NEW)
export interface ClipboardEntryDetail {
  id: string
  content: string // Full content
  content_type: string
  size_bytes: number
  is_favorited: boolean
  updated_at: number
  active_time: number
}

// Update existing types
export interface ClipboardTextItem {
  display_text: string // Changed: now always shows preview
  has_detail: boolean // NEW: replaced is_truncated
  size: number
}

// Keep other interfaces unchanged...
export interface ClipboardImageItem {
  /* unchanged */
}
export interface ClipboardFileItem {
  /* unchanged */
}
// etc.
```

**Step 2: Add API function**

```typescript
// src/api/clipboardItems.ts

/**
 * Get clipboard entry detail (full content)
 * 获取剪切板条目详情
 */
export async function getClipboardEntryDetail(id: string): Promise<ClipboardEntryDetail> {
  try {
    return await invoke('get_clipboard_entry_detail', { entryId: id })
  } catch (error) {
    console.error('Failed to get clipboard entry detail:', error)
    throw error
  }
}
```

**Step 3: Run TypeScript check**

Run: `bun run build` (TypeScript check included)
Expected: May have type errors in components (fix in next task)

**Step 4: Commit**

```bash
git add src/api/clipboardItems.ts
git commit -m "feat: add frontend types for preview/detail split

- Add ClipboardEntryDetail type
- Add getClipboardEntryDetail() API function
- Replace is_truncated with has_detail

Related: #clipboard-preview-detail"
```

---

## Task 9: Update getClipboardItems to Use Preview

**Files:**

- Modify: `src/api/clipboardItems.ts`

**Step 1: Update transformation logic**

```typescript
// src/api/clipboardItems.ts

export async function getClipboardItems(
  _orderBy?: OrderBy,
  limit?: number,
  offset?: number,
  _filter?: Filter
): Promise<ClipboardItemResponse[]> {
  try {
    const entries = await invoke<ClipboardEntryProjection[]>('get_clipboard_entries', {
      limit: limit ?? 50,
      offset: offset ?? 0,
    })

    return entries.map(entry => ({
      id: entry.id,
      device_id: '',
      is_downloaded: true,
      is_favorited: entry.is_favorited,
      created_at: entry.captured_at,
      updated_at: entry.updated_at,
      active_time: entry.active_time,
      item: {
        text: {
          display_text: entry.preview, // Use preview directly
          has_detail: entry.has_detail, // NEW field
          size: entry.size_bytes,
        },
        image: null as unknown as ClipboardImageItem,
        file: null as unknown as ClipboardFileItem,
        link: null as unknown as ClipboardLinkItem,
        code: null as unknown as ClipboardCodeItem,
        unknown: null,
      },
    }))
  } catch (error) {
    console.error('获取剪贴板历史记录失败:', error)
    throw error
  }
}
```

**Step 2: Run TypeScript check**

Run: `bun run build`
Expected: May have errors in ClipboardItem component (fix in next task)

**Step 3: Commit**

```bash
git add src/api/clipboardItems.ts
git commit -m "feat: use preview field from backend

Map backend preview to display_text, has_detail for expandability.

Related: #clipboard-preview-detail"
```

---

## Task 10: Update ClipboardItem Component for Expand

**Files:**

- Modify: `src/components/clipboard/ClipboardItem.tsx`

**Step 1: Add state and detail fetching**

```typescript
// src/components/clipboard/ClipboardItem.tsx

import { getClipboardEntryDetail, ClipboardEntryDetail } from '@/api/clipboardItems'
import { toast } from '@/components/ui/sonner'

interface ClipboardItemProps {
  // ... existing props ...
  entryId: string  // NEW: need entry ID for detail fetch
}

const ClipboardItem: React.FC<ClipboardItemProps> = ({
  index,
  type,
  time,
  content,
  entryId,  // NEW
  isSelected = false,
  onSelect,
  fileSize,
}) => {
  const { t } = useTranslation()
  const [isExpanded, setIsExpanded] = useState(false)
  const [detailContent, setDetailContent] = useState<string | null>(null)
  const [isLoadingDetail, setIsLoadingDetail] = useState(false)

  // Determine if expand button should show
  const shouldShowExpandButton = (): boolean => {
    if (!content) return false

    switch (type) {
      case 'text':
        return (content as ClipboardTextItem).has_detail === true
      case 'image':
        return true
      case 'code':
        return (content as ClipboardCodeItem).code.split('\n').length > 6
      case 'link':
      case 'file':
      default:
        return false
    }
  }

  // Handle expand toggle
  const handleExpand = async () => {
    if (isExpanded) {
      // Already expanded: collapse
      setIsExpanded(false)
    } else if (detailContent) {
      // Have cached detail: expand directly
      setIsExpanded(true)
    } else {
      // First expand: fetch detail
      setIsLoadingDetail(true)
      try {
        const detail: ClipboardEntryDetail = await getClipboardEntryDetail(entryId)
        setDetailContent(detail.content)
        setIsExpanded(true)
      } catch (e) {
        console.error('Failed to load detail:', e)
        toast.error(t('clipboard.errors.loadDetailFailed'), {
          description: e instanceof Error ? e.message : t('clipboard.errors.unknown')
        })
      } finally {
        setIsLoadingDetail(false)
      }
    }
  }

  // Update renderContent to use detail when expanded
  const renderContent = () => {
    switch (type) {
      case 'text': {
        const textItem = content as ClipboardTextItem
        const textToShow = isExpanded && detailContent ? detailContent : textItem.display_text

        return (
          <p
            className={cn(
              'whitespace-pre-wrap font-mono text-sm leading-relaxed text-foreground/90 wrap-break-word',
              !isExpanded && 'line-clamp-5'
            )}
          >
            {isLoadingDetail ? t('clipboard.item.loading') : textToShow}
          </p>
        )
      }
      // ... other cases unchanged ...
    }
  }

  // Update expand button handler
  return (
    {/* ... existing JSX ... */}
    {shouldShowExpandButton() && (
      <div
        className="flex items-center gap-1 cursor-pointer hover:text-foreground transition-colors px-2 py-1 rounded-md hover:bg-muted/50"
        onClick={e => {
          e.stopPropagation()
          void handleExpand()  // UPDATED: call async handler
        }}
      >
        {isLoadingDetail ? (
          <span>{t('clipboard.item.loading')}</span>
        ) : (
          <>
            {isExpanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
            <span>{isExpanded ? t('clipboard.item.collapse') : t('clipboard.item.expand')}</span>
          </>
        )}
      </div>
    )}
    {/* ... */}
  )
}
```

**Step 2: Update i18n strings**

```json
// src/i18n/locales/en-US.json

{
  "clipboard": {
    "item": {
      "loading": "Loading...",
      "loadDetailFailed": "Failed to load details"
    }
  }
}

// src/i18n/locales/zh-CN.json

{
  "clipboard": {
    "item": {
      "loading": "加载中...",
      "loadDetailFailed": "加载详情失败"
    }
  }
}
```

**Step 3: Update parent to pass entryId**

```typescript
// src/components/clipboard/ClipboardContent.tsx

// In the map:
<ClipboardItem
  key={item.id}
  index={index + 1}
  type={item.type}
  time={item.time}
  device={item.device}
  content={item.content}
  entryId={item.id}  // NEW: pass entryId
  isSelected={selectedIds.has(item.id)}
  onSelect={e => handleSelect(item.id, index, e)}
/>
```

**Step 4: Run TypeScript check and dev**

Run: `bun run build` then `bun tauri dev`
Expected: No type errors, expand button works

**Step 5: Commit**

```bash
git add src/components/clipboard/ClipboardItem.tsx
git add src/components/clipboard/ClipboardContent.tsx
git add src/i18n/locales/en-US.json
git add src/i18n/locales/zh-CN.json
git commit -m "feat: fetch detail on expand in ClipboardItem

- Add detail fetching state
- Show expand button based on has_detail
- Fetch full content from backend on first expand
- Cache detail in component state

Related: #clipboard-preview-detail"
```

---

## Task 11: Integration Testing

**Files:**

- Test: Manual testing in dev environment

**Step 1: Test small text entry (< 500 chars)**

1. Copy text: "hello world"
2. Run `bun tauri dev`
3. Check: Entry displays full text, NO expand button (has_detail=false)
4. Expected: ✅ Full text shown, no expand button

**Step 2: Test large text entry (> 16KB)**

1. Copy large text: Generate 20KB text file and copy
2. Check list: Shows first 500 chars, expand button visible
3. Click expand: Loading spinner → full content shown
4. Click collapse: Returns to 500-char preview
5. Expected: ✅ Preview/expand/collapse all work

**Step 3: Test large image**

1. Copy large image: > 16KB image file
2. Check list: Shows "Image (XXXXX bytes)" placeholder
3. Click expand: Loading spinner → full image shown
4. Expected: ✅ Placeholder in list, full image on expand

**Step 4: Test error handling**

1. Expand entry, then delete it from DB while expanded
2. Try expanding again
3. Expected: ✅ Toast error shown, entry remains in preview state

**Step 5: Commit**

```bash
git commit --allow-empty -m "test: verify preview/detail integration

Manual tests:
- Small text: no expand button
- Large text: preview + expand works
- Large image: placeholder + expand works
- Error handling: toast on failure

All tests passing.

Related: #clipboard-preview-detail"
```

---

## Task 12: Documentation

**Files:**

- Update: `docs/architecture/clipboard-storage.md` (if exists)
- Or: Create design doc

**Step 1: Document the preview/detail architecture**

```markdown
# Clipboard Preview/Detail Architecture

## Overview

Clipboard entries are stored with preview and detail separation for performance:

- **List view**: Returns only inline preview (no blob reads)
- **Detail view**: Fetches full content from blob on-demand

## Storage Strategy

| Content Type | Size   | inline_data     | blob_id |
| ------------ | ------ | --------------- | ------- |
| Text         | ≤ 16KB | Full content    | None    |
| Text         | > 16KB | First 500 chars | Blob ID |
| Image        | ≤ 16KB | Full data       | None    |
| Image        | > 16KB | None            | Blob ID |

## API Usage

### Get Entries List

\`\`\`typescript
const entries = await getClipboardItems({ limit: 50 })
// Each entry has: { id, preview, has_detail, ... }
// preview is safe to display without fetching detail
\`\`\`

### Get Entry Detail

\`\`\`typescript
if (entry.has_detail) {
const detail = await getClipboardEntryDetail(entry.id)
// detail.content contains full content
}
\`\`\`

## Backend Implementation

See `src-tauri/crates/uc-infra/src/clipboard/materializer.rs` for storage logic.
See `src-tauri/crates/uc-tauri/src/commands/clipboard.rs` for API commands.
```

**Step 2: Commit**

```bash
git add docs/
git commit -m "docs: add clipboard preview/detail architecture docs

Document storage strategy and API usage.

Related: #clipboard-preview-detail"
```

---

## Summary

This plan implements preview/detail separation for clipboard entries in 12 tasks:

**Backend (Rust):**

1. Text detection helper
2. Preview truncation helper
3. Materializer stores preview for large text
4. Update models (has_detail field)
5. Modify get_clipboard_entries
6. Create GetEntryDetail UseCase
7. Add get_clipboard_entry_detail command

**Frontend (TypeScript/React):** 8. Update types (ClipboardEntryDetail, has_detail) 9. Update getClipboardItems transformation 10. ClipboardItem component expand logic

**Testing & Docs:** 11. Integration testing 12. Documentation

**Performance Impact:**

- List queries: No blob reads (only inline_data)
- Detail fetch: On-demand, single blob read per expanded entry

**Estimated completion time:** 2-3 hours
