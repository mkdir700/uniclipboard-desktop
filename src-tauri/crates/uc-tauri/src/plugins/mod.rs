//! Platform-specific plugins
//!
//! This module contains platform-specific Tauri plugins.

// Platform-specific plugins
#[cfg(target_os = "macos")]
pub mod mac_rounded_corners;

/// Enable rounded corners on macOS
#[cfg(target_os = "macos")]
pub use mac_rounded_corners::{enable_modern_window_style, enable_rounded_corners, reposition_traffic_lights};
