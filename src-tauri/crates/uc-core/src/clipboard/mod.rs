//! Clipboard domain models.
mod decision;
mod entry;
mod event;
mod hash;
mod mime;
mod origin;
mod policy;
mod system;
mod timestamp;

pub use entry::*;
pub use event::*;
pub use policy::ClipboardSelection;
pub use policy::*;
pub use system::{SystemClipboardRepresentation, SystemClipboardSnapshot};

pub use decision::{ClipboardContentActionDecision, DuplicationHint, RejectReason};
pub use hash::{ContentHash, HashAlgorithm};
pub use mime::MimeType;
pub use origin::ClipboardOrigin;
pub use timestamp::TimestampMs;
