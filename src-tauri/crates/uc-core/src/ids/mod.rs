//! ID type wrappers for type safety.

pub mod blob_id;
mod clipboard;
mod id_macro;
pub mod peer_id;
pub mod rep_id;
pub mod session_id;

pub use blob_id::BlobId;
pub use clipboard::*;
pub use peer_id::PeerId;
pub use rep_id::RepresentationId;
pub use session_id::SessionId;

// re-export
pub use crate::device::value_objects::DeviceId;
