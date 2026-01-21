pub mod get_entry_detail;
pub mod get_entry_resource;
pub mod list_entry_projections;
pub mod restore_clipboard_selection;

pub use list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
