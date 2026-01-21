mod clipboard_entry_repository;
mod clipboard_event_repository;
mod clipboard_selection_repository;
mod local_clipboard;
mod payload_resolver;
mod platform_clipboard;
mod representation_cache;
mod representation_normalizer;
mod representation_repository;
mod select_representation_policy;
mod selection_resolver;
mod spool_queue;
mod thumbnail_generator;
mod thumbnail_repository;

pub use clipboard_entry_repository::ClipboardEntryRepositoryPort;
pub use clipboard_event_repository::ClipboardEventRepositoryPort;
pub use clipboard_selection_repository::ClipboardSelectionRepositoryPort;
pub use local_clipboard::SystemClipboardPort;
pub use payload_resolver::{ClipboardPayloadResolverPort, ResolvedClipboardPayload};
pub use platform_clipboard::PlatformClipboardPort;
pub use representation_cache::RepresentationCachePort;
pub use representation_normalizer::ClipboardRepresentationNormalizerPort;
pub use representation_repository::{
    ClipboardRepresentationRepositoryPort, ProcessingUpdateOutcome,
};
pub use select_representation_policy::SelectRepresentationPolicyPort;
pub use selection_resolver::SelectionResolverPort;
pub use spool_queue::{SpoolQueuePort, SpoolRequest};
pub use thumbnail_generator::{GeneratedThumbnail, ThumbnailGeneratorPort};
pub use thumbnail_repository::ThumbnailRepositoryPort;
