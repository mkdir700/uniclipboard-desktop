//! Tauri command handlers
//!
//! This module contains all Tauri command implementations.
//! Each command file corresponds to a feature area:
//! - clipboard_items: Clipboard history management
//! - p2p: Peer-to-peer networking commands
//! - setting: Application settings
//! - encryption: Encryption password management
//! - onboarding: First-run experience
//! - autostart: Auto-start configuration
//! - vault: Vault management
//! - event: Event subscription management

// Command modules - placeholders for migration from src/api/
pub mod clipboard_items;
pub mod p2p;
pub mod setting;
pub mod encryption;
pub mod onboarding;
pub mod autostart;
pub mod vault;
pub mod event;
