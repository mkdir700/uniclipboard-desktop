pub mod clipboard;
#[cfg(target_os = "windows")]
mod utils;
mod local;
pub use local::LocalClipboard;
pub use clipboard::{RsClipboard, RsClipboardChangeHandler};
