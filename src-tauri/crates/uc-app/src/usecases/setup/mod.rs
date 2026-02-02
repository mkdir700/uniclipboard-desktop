//! Setup use cases.
//!
//! This module exposes the setup orchestrator.

mod context;
pub mod orchestrator;

pub use orchestrator::{SetupOrchestrator, SetupOrchestratorError};
