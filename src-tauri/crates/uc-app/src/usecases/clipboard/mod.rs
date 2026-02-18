pub mod get_entry_detail;
pub mod get_entry_resource;
pub mod list_entry_projections;
pub mod resolve_blob_resource;
pub mod resolve_thumbnail_resource;
pub mod restore_clipboard_selection;
pub mod sync_inbound;
pub mod sync_outbound;
pub mod touch_clipboard_entry;

pub use list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
