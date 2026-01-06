//! Business logic use cases
//! 是否是独立 Use Case，
//! 取决于“是否需要用户 / 系统再次做出决策”

pub mod clipboard;
pub mod pair_device;
pub mod start_app;
pub mod sync_clipboard;

pub use pair_device::PairDevice;
pub use start_app::{AppContext, StartApp};
pub use sync_clipboard::SyncClipboard;
