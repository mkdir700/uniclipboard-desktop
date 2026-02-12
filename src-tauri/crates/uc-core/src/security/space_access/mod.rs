pub mod action;
pub mod domain;
pub mod error;
pub mod event;
pub mod reason_codec;
pub mod state;
pub mod state_machine;

pub use domain::SpaceAccessProofArtifact;
pub use reason_codec::{
    deny_reason_from_code, deny_reason_to_code, DENY_REASON_EXPIRED, DENY_REASON_INTERNAL_ERROR,
    DENY_REASON_INVALID_PROOF, DENY_REASON_SESSION_MISMATCH, DENY_REASON_SPACE_MISMATCH,
};
