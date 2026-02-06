//! Setup domain module.
//!
//! This module defines the setup state machine types.

pub mod action;
pub mod error;
pub mod event;
pub mod state;
pub mod state_machine;
pub mod status;

pub use action::SetupAction;
pub use error::SetupError;
pub use event::SetupEvent;
pub use state::SetupState;
pub use state_machine::SetupStateMachine;
pub use status::SetupStatus;
