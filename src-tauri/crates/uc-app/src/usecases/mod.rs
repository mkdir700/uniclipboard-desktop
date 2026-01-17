//! Business logic use cases
//! 是否是独立 Use Case，
//! 取决于"是否需要用户 / 系统再次做出决策"
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
pub mod auto_unlock_encryption_session;
pub mod change_passphrase;
pub mod clipboard;
pub mod delete_clipboard_entry;
pub mod get_settings;
pub mod initialize_encryption;
pub mod internal;
pub mod is_encryption_initialized;
pub mod list_clipboard_entries;
pub mod list_clipboard_entry_previews;
pub mod onboarding;
pub mod settings;
pub mod start_clipboard_watcher;
pub mod update_settings;

pub use auto_unlock_encryption_session::AutoUnlockEncryptionSession;
pub use clipboard::list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
pub use delete_clipboard_entry::DeleteClipboardEntry;
pub use get_settings::GetSettings;
pub use initialize_encryption::InitializeEncryption;
pub use is_encryption_initialized::IsEncryptionInitialized;
pub use list_clipboard_entries::ListClipboardEntries;
pub use start_clipboard_watcher::StartClipboardWatcher;
pub use update_settings::UpdateSettings;

// Re-export onboarding types for Tauri command serialization
pub use onboarding::CompleteOnboarding;
pub use onboarding::GetOnboardingState;
pub use onboarding::InitializeOnboarding;
pub use onboarding::OnboardingStateDto;
