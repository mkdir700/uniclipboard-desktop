//! Business logic use cases

pub mod pair_device;
pub mod start_app;
pub mod sync_clipboard;

pub use pair_device::PairDevice;
pub use start_app::{StartApp, AppContext};
pub use sync_clipboard::SyncClipboard;
