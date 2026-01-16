//! # uc-platform
//!
//! Platform-specific implementations for UniClipboard.
//!
//! This crate contains infrastructure implementations that interact with
//! the operating system, external services, and hardware.

// Tracing support for platform layer instrumentation
pub use tracing;

pub mod adapters;
pub mod bootstrap;
pub mod capability;
pub mod clipboard;
pub mod file_keyring;
pub mod ipc;
pub mod key_scope;
pub mod keyring;
pub mod ports;
pub mod runtime;
pub mod secure_storage;
