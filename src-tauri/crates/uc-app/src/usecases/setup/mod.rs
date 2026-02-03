//! /// Setup is a business phase.
//! It is the only authority to decide whether the app is initialized.
/// Do NOT infer setup progress from encryption / pairing state.
mod context;
pub mod orchestrator;

pub use orchestrator::{SetupError, SetupOrchestrator};
