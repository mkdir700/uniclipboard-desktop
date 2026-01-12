//! UniClipboard Application Orchestration Layer
//!
//! This crate contains business logic use cases and runtime orchestration.

pub mod bootstrap;
pub mod builder;
pub mod models;
pub mod ports;
pub mod usecases;

pub use builder::{App, AppBuilder};
pub use models::ClipboardEntryProjection;
