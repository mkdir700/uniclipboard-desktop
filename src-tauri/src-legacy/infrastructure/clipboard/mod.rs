pub mod clipboard;
mod local;
mod traits;
#[cfg(target_os = "windows")]
mod utils;
pub use local::LocalClipboard;
