//! UniClipboard Application Orchestration Layer
//!
//! This crate contains business logic use cases and runtime orchestration.

pub mod bootstrap;
pub mod event;
pub mod use_cases;

pub use event::AppEvent;
