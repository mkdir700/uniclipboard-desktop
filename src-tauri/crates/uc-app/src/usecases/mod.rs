//! Business logic use cases
//! 是否是独立 Use Case，
//! 取决于“是否需要用户 / 系统再次做出决策”
//!
//! [ClipboardWatcher]
//        ↓
// CaptureClipboardUseCase
//         ↓
// ---------------------------------
//         ↓
// ListClipboardEntryPreviewsUseCase  → UI 列表
// GetClipboardEntryPreviewUseCase    → UI hover / detail
// ---------------------------------
//         ↓
// MaterializeClipboardSelectionUseCase → 粘贴 / 恢复 / 同步

pub mod change_passphrase;
pub mod clipboard;
pub mod initialize_encryption;
pub mod internal;
pub mod list_clipboard_entries;
pub mod list_clipboard_entry_previews;
pub mod settings;

pub use list_clipboard_entries::ListClipboardEntries;
