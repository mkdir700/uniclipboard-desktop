pub mod local_clipboard_trait;  
pub mod remote_sync_trait;
pub mod storage_trait;
pub mod sync_provider_trait;

pub use local_clipboard_trait::LocalClipboardTrait;
pub use remote_sync_trait::{RemoteClipboardSync, RemoteSyncManagerTrait};