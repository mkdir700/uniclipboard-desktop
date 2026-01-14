//! Platform-specific Tauri command modules
//! 平台特定的 Tauri 命令模块

#[cfg(target_os = "macos")]
pub mod mac_rounded_corners;

// Re-export macOS commands for invoke_handler macro
#[cfg(target_os = "macos")]
pub use mac_rounded_corners::{
    enable_rounded_corners,
    enable_modern_window_style,
    reposition_traffic_lights,
};
