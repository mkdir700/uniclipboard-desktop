# Frontend API Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate frontend from legacy Tauri commands to new hexagonal architecture commands, fixing clipboard list display and event monitoring.

**Architecture:**

- Backend: Add `AppHandle` to `AppRuntime` for event sending, supplement `ClipboardEntryProjection` fields
- Frontend: Adapt to new command names (`get_clipboard_entries`, `get_settings`) and event format (`clipboard://event`)
- No backward compatibility - direct switch to new architecture

**Tech Stack:**

- Rust: Tauri 2, serde, tokio, async-trait
- Frontend: React 18, TypeScript, Tauri API, Redux Toolkit

---

## Task 1: Add AppHandle to AppRuntime

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Add AppHandle field to AppRuntime struct**

Add `app_handle` field to the `AppRuntime` struct at line 68:

```rust
pub struct AppRuntime {
    /// Application dependencies
    pub deps: AppDeps,
    /// Tauri AppHandle for emitting events (optional, set after Tauri setup)
    app_handle: Option<tauri::AppHandle>,
}
```

**Step 2: Update AppRuntime::new() constructor**

Modify the `new()` method at line 76:

```rust
pub fn new(deps: AppDeps) -> Self {
    Self {
        deps,
        app_handle: None,
    }
}
```

**Step 3: Add setter method for app_handle**

Add this method after the `new()` method:

```rust
/// Set the Tauri AppHandle for event emission.
/// This must be called after Tauri setup completes.
pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
    self.app_handle = Some(handle);
}

/// Get a reference to the AppHandle, if available.
pub fn app_handle(&self) -> Option<&tauri::AppHandle> {
    self.app_handle.as_ref()
}
```

**Step 4: Update imports**

Add tauri import at the top of the file (around line 32):

```rust
use uc_app::{App, AppDeps};
use uc_core::config::AppConfig;
use uc_core::ports::ClipboardChangeHandler;
use uc_core::SystemClipboardSnapshot;
use tauri::AppHandle;  // NEW
```

**Step 5: Verify compilation**

Run: `cd src-tauri && cargo check --message-format=json 2>&1 | grep -E "(error|warning)" | head -20`

Expected: No errors about missing fields or methods

**Step 6: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat: add AppHandle to AppRuntime for event emission"
```

---

## Task 2: Emit clipboard events in on_clipboard_changed

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Import ClipboardEvent**

Add import at the top of the file (around line 32):

```rust
use uc_tauri::events::ClipboardEvent;
```

**Step 2: Modify on_clipboard_changed to emit events**

Replace the entire `on_clipboard_changed` method implementation (lines 426-448) with:

```rust
async fn on_clipboard_changed(&self, snapshot: SystemClipboardSnapshot) -> anyhow::Result<()> {
    // Create CaptureClipboardUseCase with dependencies
    let usecase = uc_app::usecases::internal::capture_clipboard::CaptureClipboardUseCase::new(
        self.deps.clipboard.clone(),
        self.deps.clipboard_entry_repo.clone(),
        self.deps.clipboard_event_repo.clone(),
        self.deps.representation_policy.clone(),
        self.deps.representation_materializer.clone(),
        self.deps.device_identity.clone(),
    );

    // Execute capture with the provided snapshot
    match usecase.execute_with_snapshot(snapshot).await {
        Ok(event_id) => {
            tracing::debug!("Successfully captured clipboard, event_id: {}", event_id);

            // Emit event to frontend if AppHandle is available
            if let Some(app) = &self.app_handle {
                let event = ClipboardEvent::NewContent {
                    entry_id: event_id.to_string(),
                    preview: "New clipboard content".to_string(),
                };

                if let Err(e) = app.emit("clipboard://event", event) {
                    tracing::warn!("Failed to emit clipboard event to frontend: {}", e);
                } else {
                    tracing::debug!("Successfully emitted clipboard://event to frontend");
                }
            } else {
                tracing::debug!("AppHandle not available, skipping event emission");
            }

            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to capture clipboard: {:?}", e);
            Err(e)
        }
    }
}
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check --message-format=json 2>&1 | grep -E "error" | head -10`

Expected: No compilation errors

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "feat: emit clipboard://event on clipboard change"
```

---

## Task 3: Inject AppHandle in main.rs

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Modify runtime_for_tauri to be mutable**

Change line 161 from:

```rust
let runtime_for_tauri = runtime_for_handler.clone();
```

To:

```rust
let mut runtime_for_tauri = runtime_for_handler.clone();
```

**Step 2: Inject AppHandle in setup block**

Add this code inside the `.setup(move |app_handle| {` block, before `Ok(())` (around line 263):

```rust
// Inject AppHandle into runtime for event emission
runtime_for_tauri.set_app_handle(app_handle.clone());
log::info!("AppHandle injected into AppRuntime");
```

**Step 3: Update manage() call**

We need to use `Arc::make_mut()` to modify the Arc contents. Replace lines 181-182:

```rust
// Manage Arc<AppRuntime> for use case access
// NOTE: Commands need to use State<'_, Arc<AppRuntime>> instead of State<'_, AppRuntime>
.manage(runtime_for_tauri)
```

With:

```rust
// Clone Arc for Tauri state management (original is used for clipboard handler)
.manage(runtime_for_handler.clone())
```

**Step 4: Remove runtime_for_tauri variable**

Since we're now using `runtime_for_handler` directly, remove the line that creates `runtime_for_tauri` (line 161).

**Step 5: Verify compilation**

Run: `cd src-tauri && cargo check --message-format=json 2>&1 | grep -E "error" | head -10`

Expected: No compilation errors

**Step 6: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: inject AppHandle into AppRuntime during Tauri setup"
```

---

## Task 4: Add fields to ClipboardEntryProjection

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/models/mod.rs`

**Step 1: Add missing fields to ClipboardEntryProjection**

Replace the entire struct (lines 16-27) with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryProjection {
    /// Unique identifier for the entry
    pub id: String,
    /// Preview text for display
    pub preview: String,
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
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check --message-format=json 2>&1 | grep -E "error" | head -10`

Expected: Error about missing fields in the projection creation

**Step 3: Fix projection creation**

Find where `ClipboardEntryProjection` is created and add the new fields. Run:

```bash
cd src-tauri && rg "ClipboardEntryProjection \{" -A 3
```

Update each creation site to include default values for new fields:

```rust
is_favorited: false,
updated_at: captured_at,  // Same as captured_at initially
active_time: captured_at,  // Same as captured_at initially
```

**Step 4: Verify compilation**

Run: `cd src-tauri && cargo check --message-format=json 2>&1 | grep -E "error" | head -10`

Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/models/mod.rs
git commit -m "feat: add is_favorited, updated_at, active_time to ClipboardEntryProjection"
```

---

## Task 5: Update frontend clipboardItems.ts API

**Files:**

- Modify: `src/api/clipboardItems.ts`

**Step 1: Change getClipboardItems command name**

Replace the `getClipboardItems` function implementation (lines 103-120) with:

```typescript
export async function getClipboardItems(
  orderBy?: OrderBy,
  limit?: number,
  offset?: number,
  filter?: Filter
): Promise<ClipboardItemResponse[]> {
  try {
    // Map Filter enum to backend format if needed
    const mappedFilter = filter === Filter.All ? undefined : filter

    // Use new command name: get_clipboard_entries
    const entries = await invoke<ClipboardEntryProjection[]>('get_clipboard_entries', {
      limit: limit ?? 50,
      offset: offset ?? 0,
    })

    // Transform backend projection to frontend response format
    return entries.map(entry => ({
      id: entry.id,
      device_id: '', // Not in projection yet, use empty string
      is_downloaded: true, // Default to true for local entries
      is_favorited: entry.is_favorited,
      created_at: entry.captured_at,
      updated_at: entry.updated_at,
      active_time: entry.active_time,
      item: {
        text:
          entry.content_type === 'text'
            ? {
                display_text: entry.preview,
                is_truncated: entry.preview.length > 100,
                size: entry.preview.length,
              }
            : (null as unknown as ClipboardTextItem),
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

**Step 2: Add ClipboardEntryProjection type**

Add this interface after the imports (around line 262):

```typescript
// Backend projection type
interface ClipboardEntryProjection {
  id: string
  preview: string
  captured_at: number
  content_type: string
  is_encrypted: boolean
  is_favorited: boolean
  updated_at: number
  active_time: number
}
```

**Step 3: Change deleteClipboardItem command name**

Replace the `deleteClipboardItem` function (lines 145-152) with:

```typescript
export async function deleteClipboardItem(id: string): Promise<boolean> {
  try {
    return await invoke('delete_clipboard_entry', { entry_id: id })
  } catch (error) {
    console.error('删除剪贴板条目失败:', error)
    throw error
  }
}
```

**Step 4: Remove unused functions (optional cleanup)**

Remove these functions that reference non-existent commands:

- `getClipboardStats`
- `getClipboardItem`
- `clearClipboardItems`
- `syncClipboardItems`
- `copyClipboardItem`
- `favoriteClipboardItem`
- `unfavoriteClipboardItem`

**Step 5: Verify TypeScript compilation**

Run: `bun run build`

Expected: No TypeScript errors

**Step 6: Commit**

```bash
git add src/api/clipboardItems.ts
git commit -m "feat: migrate clipboardItems API to new backend commands"
```

---

## Task 6: Update frontend SettingContext

**Files:**

- Modify: `src/contexts/SettingContext.tsx`

**Step 1: Update loadSetting to use get_settings**

Replace the `loadSetting` function implementation (lines 25-39) with:

```typescript
const loadSetting = async () => {
  try {
    setLoading(true)
    // New command returns JSON object directly, no parsing needed
    const settingObj = await invoke<Setting>('get_settings')
    setSetting(settingObj)
    setError(null)
  } catch (err) {
    console.error('加载设置失败:', err)
    setError(`加载设置失败: ${err}`)
  } finally {
    setLoading(false)
  }
}
```

**Step 2: Update saveSetting to use update_settings**

Replace the `saveSetting` function (lines 42-55) with:

```typescript
const saveSetting = async (newSetting: Setting) => {
  try {
    setLoading(true)
    // New command: update_settings, takes JSON object directly
    await invoke('update_settings', { settings: newSetting })
    setSetting(newSetting)
    setError(null)
  } catch (err) {
    console.error('保存设置失败:', err)
    setError(`保存设置失败: ${err}`)
    throw err
  } finally {
    setLoading(false)
  }
}
```

**Step 3: Remove JSON.stringify from setting update calls**

The `update_settings` command takes a JSON object directly. Remove `JSON.stringify()` from all update functions.

Update `updateGeneralSetting` (lines 63-73), `updateSyncSetting` (lines 76-86), etc. - they already call `saveSetting` which handles the conversion, so no changes needed there.

**Step 4: Verify TypeScript compilation**

Run: `bun run build`

Expected: No TypeScript errors

**Step 5: Commit**

```bash
git add src/contexts/SettingContext.tsx
git commit -m "feat: migrate settings API to new backend commands"
```

---

## Task 7: Update DashboardPage to use clipboard://event

**Files:**

- Modify: `src/pages/DashboardPage.tsx`

**Step 1: Add ClipboardEvent type**

Add this interface after the imports (around line 12):

```typescript
// Backend clipboard event type
interface ClipboardEvent {
  type: 'NewContent' | 'Deleted'
  entry_id?: string
  preview?: string
}
```

**Step 2: Remove legacy listen_clipboard_new_content invocation**

Remove these lines from the `setupListener` function (lines 100-103):

```typescript
// REMOVE THIS BLOCK:
console.log(t('dashboard.logs.startingBackendListener'))
await invoke('listen_clipboard_new_content')
console.log(t('dashboard.logs.backendListenerStarted'))
```

**Step 3: Update event listener to use clipboard://event**

Replace the `listen` call (lines 114-135) with:

```typescript
// Listen to new clipboard://event format
const unlisten = await listen<{ type: string; entry_id?: string; preview?: string }>(
  'clipboard://event',
  event => {
    console.log(t('dashboard.logs.newClipboardEvent'), event)

    // Check event type
    if (event.payload.type === 'NewContent' && event.payload.entry_id) {
      // Check event timestamp to avoid processing duplicate events within short time
      const currentTime = Date.now()
      if (
        globalListenerState.lastEventTimestamp &&
        currentTime - globalListenerState.lastEventTimestamp < DEBOUNCE_DELAY
      ) {
        console.log(t('dashboard.logs.ignoringDuplicateEvent'))
        return
      }

      // Update last event timestamp
      globalListenerState.lastEventTimestamp = currentTime

      // Use debounced function to load data
      debouncedLoadData(currentFilterRef.current)
    }
  }
)
```

**Step 4: Remove unused invoke import**

Since we're no longer using `invoke('listen_clipboard_new_content')`, remove the `invoke` import from line 1 if it's not used elsewhere in the file.

**Step 5: Verify TypeScript compilation**

Run: `bun run build`

Expected: No TypeScript errors

**Step 6: Commit**

```bash
git add src/pages/DashboardPage.tsx
git commit -m "feat: migrate DashboardPage to clipboard://event"
```

---

## Task 8: Create ClipboardEvent types file

**Files:**

- Create: `src/types/events.ts`

**Step 1: Create events type file**

```typescript
/**
 * Clipboard events from backend
 */
export interface ClipboardEvent {
  type: 'NewContent' | 'Deleted'
  entry_id?: string
  preview?: string
}

/**
 * Setting changed event data
 */
export interface SettingChangedEvent {
  settingJson: string
  timestamp: number
}
```

**Step 2: Update DashboardPage imports**

Change line 12 in `src/pages/DashboardPage.tsx` from:

```typescript
import { toast } from '@/components/ui/sonner'
```

To:

```typescript
import { toast } from '@/components/ui/sonner'
import { ClipboardEvent } from '@/types/events'
```

**Step 3: Update SettingContext imports**

Change line 5 in `src/contexts/SettingContext.tsx` from:

```typescript
import { SettingContext, type SettingContextType, type Setting } from '@/types/setting'
```

To:

```typescript
import { SettingContext, type SettingContextType, type Setting } from '@/types/setting'
import type { SettingChangedEvent } from '@/types/events'
```

Remove the local `SettingChangedEvent` interface definition (lines 7-11).

**Step 4: Verify TypeScript compilation**

Run: `bun run build`

Expected: No TypeScript errors

**Step 5: Commit**

```bash
git add src/types/events.ts src/pages/DashboardPage.tsx src/contexts/SettingContext.tsx
git commit -m "feat: create shared events type file"
```

---

## Task 9: Manual testing

**Step 1: Start development server**

Run: `bun tauri dev`

Expected: Application starts without console errors

**Step 2: Verify clipboard capture**

1. Copy some text in another app (e.g., "Hello World")
2. Check the application's clipboard list
3. Verify the new entry appears
4. Check browser console for "clipboard://event" events

Expected: New clipboard entry appears in list, console shows event received

**Step 3: Verify settings load**

1. Open Settings page
2. Verify settings are displayed correctly
3. Change a setting
4. Verify it saves

Expected: Settings load and save without errors

**Step 4: Verify delete functionality**

1. Click on a clipboard entry
2. Delete it
3. Verify it's removed from the list

Expected: Entry is deleted successfully

**Step 5: Check for remaining errors**

Check browser console for any remaining errors like:

- "Command X not found"
- "Failed to invoke Y"
- Network errors

**Step 6: Document any remaining issues**

If there are remaining issues, create a new design document to address them.

---

## Migration Status Checklist

- [x] Task 1: Add AppHandle to AppRuntime
- [x] Task 2: Emit clipboard events in on_clipboard_changed
- [x] Task 3: Inject AppHandle in main.rs
- [x] Task 4: Add fields to ClipboardEntryProjection
- [x] Task 5: Update frontend clipboardItems.ts API
- [x] Task 6: Update frontend SettingContext
- [x] Task 7: Update DashboardPage to use clipboard://event
- [x] Task 8: Create ClipboardEvent types file
- [x] Task 9: Manual testing

---

## References

- Design document: `docs/plans/2026-01-15-frontend-api-migration-design.md`
- Event forwarding: `src-tauri/crates/uc-tauri/src/events/mod.rs`
- Clipboard handler: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:426-448`
- Commands list: `src-tauri/src/main.rs:110-134`
