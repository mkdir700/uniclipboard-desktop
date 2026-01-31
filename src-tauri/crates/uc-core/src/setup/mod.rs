//! Setup domain module.
//!
//! This module defines the onboarding setup state machine types.

pub mod state_machine;

pub use state_machine::{SetupAction, SetupError, SetupEvent, SetupState, SetupStateMachine};
