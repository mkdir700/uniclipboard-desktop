pub mod macos;
pub mod windows;

// macOS exports
#[cfg(target_os = "macos")]
pub use macos::MacOSClipboard as Clipboard;

// Windows exports
#[cfg(target_os = "windows")]
pub use windows::WindowsClipboard as Clipboard;
