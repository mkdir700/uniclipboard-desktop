pub mod linux;
pub mod macos;
pub mod windows;

// macOS exports
#[cfg(target_os = "macos")]
pub use macos::MacOSClipboard as LocalClipboard;

// Windows exports
#[cfg(target_os = "windows")]
pub use windows::WindowsClipboard as LocalClipboard;

// Unix exports
#[cfg(target_os = "linux")]
pub use linux::LinuxClipboard as LocalClipboard;
