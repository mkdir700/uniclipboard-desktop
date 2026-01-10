//! Clipboard domain models.
mod blob;
mod clipboard_entry;
mod clipboard_event;
mod clipboard_selection;
mod content;
mod decision;
mod domain;
mod event;
pub mod meta_keys;
mod mime;
mod policy;
mod snapshot;
mod system;
mod view;

pub use blob::NewBlob;
pub use clipboard_entry::{NewClipboardEntry, NewClipboardSelection};
pub use clipboard_event::{NewClipboardEvent, NewSnapshotRepresentation};
pub use clipboard_selection::ClipboardSelection;
pub use policy::*;
pub use system::{SystemClipboardRepresentation, SystemClipboardSnapshot};

pub use content::{
    ClipboardContent, ClipboardData, ClipboardItem, ClipboardOrigin, ContentHash, ItemHash,
    PayloadHash, TimestampMs,
};
pub use decision::{ClipboardContentActionDecision, DuplicationHint, RejectReason};
pub use domain::ClipboardContentDecisionDomain;
pub use mime::MimeType;
pub use snapshot::ClipboardDecisionSnapshot;
pub use view::{ClipboardContentView, ClipboardItemView, ClipboardRecordId};
