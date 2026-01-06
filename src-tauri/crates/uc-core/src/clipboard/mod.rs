//! Clipboard domain models.
mod content;
mod decision;
mod domain;
mod event;
pub mod meta_keys;
mod mime;
mod snapshot;
mod view;

#[cfg(test)]
mod tests;

pub use content::{ClipboardContent, ClipboardData, ClipboardItem};
pub use mime::MimeType;
pub use snapshot::ClipboardDecisionSnapshot;
pub use view::{ClipboardContentView, ClipboardItemView, ClipboardOrigin, ClipboardRecordId};
