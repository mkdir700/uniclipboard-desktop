//! # uc-platform
//!
//! Platform-specific implementations for UniClipboard.
//!
//! This crate contains infrastructure implementations that interact with
//! the operating system, external services, and hardware.

pub mod bootstrap;
pub mod clipboard;
pub mod ipc;
pub mod keyring;
pub mod ports;
pub mod runtime;
pub mod tauri;
