//! # uc-tauri
//!
//! Tauri integration layer for UniClipboard.
//!
//! This crate provides:
//! - Tauri command handlers
//! - Event system integration
//! - Platform-specific plugins
//!
//! ## Modules
//!
//! - **commands**: Tauri command handlers (clipboard_items, p2p, setting, etc.)
//! - **state**: Tauri state management (event listeners)
//! - **plugins**: Platform-specific plugins (macOS rounded corners)
//! - **runtime**: AppRuntime and handle definitions

pub mod commands;
pub mod plugins;
pub mod runtime;
pub mod state;

// Re-export commonly used types
pub use runtime::AppRuntimeHandle;
pub use state::EventListenerState;
