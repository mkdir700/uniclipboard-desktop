//! UniClipboard Application Orchestration Layer
//!
//! This crate contains business logic use cases and runtime orchestration.

pub mod adapters;
pub mod use_cases;

pub use use_cases::{pair_device, start_app, sync_clipboard};
