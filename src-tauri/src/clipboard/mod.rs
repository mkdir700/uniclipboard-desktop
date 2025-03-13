pub mod clipboard;
#[cfg(target_os = "windows")]
mod utils;
mod local;
mod traits;
pub use local::LocalClipboard;
pub use clipboard::RsClipboard;
pub use clipboard::RsClipboardChangeHandler;
pub use traits::LocalClipboardTrait;