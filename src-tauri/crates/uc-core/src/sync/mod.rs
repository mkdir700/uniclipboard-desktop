//! Sync domain models and state machine.

pub mod domain;
pub mod event;
pub mod state;

pub use domain::SyncDomain;
pub use event::SyncEvent;
pub use state::SyncState;
