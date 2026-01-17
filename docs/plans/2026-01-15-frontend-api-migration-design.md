# Frontend API Migration Design

## Problem / 问题

前端调用 legacy Tauri 命令失败，因为后端已迁移到新架构（Hexagonal Architecture），命令名和数据结构发生变化。

**Errors / 错误**:

- `Command get_clipboard_items not found`
- `Command listen_clipboard_new_content not found`
- `Command get_setting not found`

## Root Cause Analysis / 根本原因分析

### Legacy Commands (Frontend expects / 前端期望)

- `get_clipboard_items` → Returns `ClipboardItemResponse[]` (full details)
- `listen_clipboard_new_content` → Event listener command
- `get_setting` → Returns JSON string
- `save_setting` → Saves settings

### New Architecture Commands (Backend provides / 后端提供)

- `get_clipboard_entries` → Returns `Vec<ClipboardEntryProjection>` (simplified)
- `clipboard://event` → Structured event via `events/mod.rs`
- `get_settings` → Returns `Value` (JSON object)
- `update_settings` → Updates settings

### Key Differences / 关键差异

1. **Command names**: `get_clipboard_items` vs `get_clipboard_entries`
2. **Data structure**: Simplified projection vs full details (text/image/file/link/code)
3. **Event mechanism**: Command-based vs Tauri event listener
4. **Settings API**: Singular vs plural, string vs object

## Solution / 解决方案

### Approach / 方案

- **Backend**: Add event sending, supplement projection fields, add details command
- **Frontend**: Adapt to new command names and event format
- **No backward compatibility** - Direct switch (前端不需要向后兼容)

### Data Flow / 数据流

```
Clipboard Capture Flow:
Platform layer detects change
    ↓
ClipboardWatcher sends PlatformEvent::ClipboardChanged
    ↓
PlatformRuntime calls ClipboardChangeHandler callback
    ↓
AppRuntime.on_clipboard_changed() captures entry
    ↓
Send event: clipboard://event (ClipboardEvent::NewContent)
    ↓
Frontend listens via Tauri listen() API
    ↓
Dashboard dispatches fetchClipboardItems()
```

### On-Demand Details Loading / 按需加载详情

```
User clicks entry
    ↓
Component calls getClipboardItemDetails(id)
    ↓
API calls get_clipboard_entry_details
    ↓
Backend returns full content (text/image/file/link/code)
    ↓
Component updates local state
```

## Implementation Plan / 实施计划

### Phase 1: Backend Core Changes / 后端核心改动

**1. Modify `AppRuntime`** (`src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`)

```rust
pub struct AppRuntime {
    pub deps: AppDeps,
    app_handle: Option<AppHandle>,  // NEW
}

impl AppRuntime {
    pub fn new(deps: AppDeps) -> Self {
        Self { deps, app_handle: None }
    }

    pub fn with_app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }
}
```

**2. Modify `on_clipboard_changed`** (same file)

```rust
async fn on_clipboard_changed(&self, snapshot: SystemClipboardSnapshot) -> anyhow::Result<()> {
    // ... capture logic ...

    match usecase.execute_with_snapshot(snapshot).await {
        Ok(event_id) => {
            tracing::debug!("Successfully captured clipboard, event_id: {}", event_id);

            // NEW: Send event to frontend
            if let Some(app) = &self.app_handle {
                let _ = app.emit("clipboard://event", ClipboardEvent::NewContent {
                    entry_id: event_id.to_string(),
                    preview: "Entry captured".to_string(),
                });
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

**3. Modify `main.rs`** (`src-tauri/src/main.rs`)

```rust
// In run_app() function, after creating runtime:
let runtime = AppRuntime::new(deps);

// After AppHandle is available in setup:
.setup(move |app_handle| {
    // ... existing code ...

    // Inject AppHandle into runtime
    runtime_for_tauri.set_app_handle(app_handle);

    Ok(())
})
```

**4. Supplement `ClipboardEntryProjection`** (`src-tauri/crates/uc-tauri/src/models/mod.rs`)

```rust
pub struct ClipboardEntryProjection {
    pub id: String,
    pub preview: String,
    pub captured_at: u64,
    pub content_type: String,
    pub is_encrypted: bool,
    // NEW fields for frontend compatibility:
    pub is_favorited: bool,
    pub updated_at: u64,
    pub active_time: u64,
}
```

### Phase 2: Frontend API Adaptation / 前端 API 适配

**1. Modify `src/api/clipboardItems.ts`**

```typescript
// Change command name
export async function getClipboardItems(
  orderBy?: OrderBy,
  limit?: number,
  offset?: number,
  filter?: Filter
): Promise<ClipboardItemResponse[]> {
  return await invoke('get_clipboard_entries', { limit, offset })
}

// Change delete command
export async function deleteClipboardItem(id: string): Promise<boolean> {
  return await invoke('delete_clipboard_entry', { entry_id: id })
}

// NEW: Get entry details
export async function getClipboardItemDetails(id: string): Promise<ClipboardItemResponse> {
  return await invoke('get_clipboard_entry_details', { entry_id: id })
}
```

**2. Add `ClipboardEvent` type** (new file or in existing types)

```typescript
export interface ClipboardEvent {
  type: 'NewContent' | 'Deleted'
  entry_id?: string
  preview?: string
}
```

**3. Modify `src/contexts/SettingContext.tsx`**

```typescript
// Change from get_setting to get_settings
const result = await invoke('get_settings') // Returns object, not string
const settingObj = result as Setting // No JSON.parse needed

// Change from save_setting to update_settings
await invoke('update_settings', { settings: newSetting })
```

**4. Modify `src/pages/DashboardPage.tsx`**

```typescript
// REMOVE: invoke('listen_clipboard_new_content')
// KEEP: Use Tauri's listen() API directly

const unlisten = await listen<ClipboardEvent>('clipboard://event', event => {
  if (event.payload.type === 'NewContent') {
    debouncedLoadData(currentFilterRef.current)
  }
})
```

### Phase 3: Details Feature / 详情功能（按需）

**Backend: Add `get_clipboard_entry_details` command**

```rust
#[tauri::command]
pub async fn get_clipboard_entry_details(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<ClipboardItemDetails, String> {
    // Fetch entry with representations
    // Return full content (text/image/file/link/code)
}
```

**Frontend: Integrate details loading**

- Call `getClipboardItemDetails(id)` when user clicks entry
- Update component local state to show details

## Testing Strategy / 测试策略

### Backend Tests

- Unit test for event sending in `on_clipboard_changed`
- Integration test: capture → event → frontend receive

### Manual Testing

1. Start app with `bun tauri dev`
2. Copy text in another app
3. Verify clipboard list refreshes
4. Verify no console errors

## Files Changed / 改动文件

### Backend (5 files)

- `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
- `src-tauri/src/main.rs`
- `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- `src-tauri/crates/uc-tauri/src/models/mod.rs`
- `src-tauri/crates/uc-tauri/src/events/mod.rs`

### Frontend (3 files)

- `src/api/clipboardItems.ts`
- `src/contexts/SettingContext.tsx`
- `src/pages/DashboardPage.tsx`

## Migration Status / 迁移状态

- [ ] Phase 1: Backend core changes
- [ ] Phase 2: Frontend API adaptation
- [ ] Phase 3: Details feature (optional)

## References / 参考

- New architecture commands: `src-tauri/src/main.rs:110-134`
- Legacy commands: `src-tauri/src-legacy/main.rs:270-320`
- Event forwarding: `src-tauri/crates/uc-tauri/src/events/mod.rs`
- Clipboard handler: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs:426-448`
