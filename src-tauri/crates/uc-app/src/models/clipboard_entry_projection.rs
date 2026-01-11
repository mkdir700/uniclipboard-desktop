use uc_core::ids::EntryId;

/// A read-only projection of a ClipboardEntry,
/// optimized for query and presentation purposes.
///
/// This is NOT a domain entity.
/// This model may change as query requirements evolve.
pub struct ClipboardEntryProjection {
    pub entry_id: EntryId,

    /// Primary human-readable summary
    pub title: String,

    /// Secondary description (mime, format, metadata)
    pub subtitle: Option<String>,

    /// Content category for consumers
    pub kind: EntryProjectionKind,

    /// Size of the projected content (not payload size)
    pub projected_size: i64,

    /// Whether this entry is ready for materialized usage
    pub ready_for_use: bool,

    /// Projection state (derived, degraded, etc.)
    pub state: ProjectionState,
}

pub enum EntryProjectionKind {
    PlainText,
    RichText,
    Image,
    File,
    Binary,
    Unknown,
}

pub enum ProjectionState {
    /// Fully derived from available data
    Ready,

    /// Derived from partial information
    Degraded,

    /// Undergoing background materialization
    Materializing,

    /// Projection failed
    Error,
}
