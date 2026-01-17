//! Use case for listing clipboard entry projections with cross-repo aggregation
//! 跨仓库聚合的剪贴板条目投影列表用例

mod list_entry_projections;

pub use list_entry_projections::{
    EntryProjectionDto, ListClipboardEntryProjections, ListProjectionsError,
};
