//! # uc-platform
//!
//! Platform-specific implementations for UniClipboard.
//!
//! This crate contains infrastructure implementations that interact with
//! the operating system, external services, and hardware.

pub mod adatpers;
pub mod ipc;
pub mod ports;
pub mod runtime;
