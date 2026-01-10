use crate::db::models::snapshot_representation::NewSnapshotRepresentationRow;
use crate::db::ports::Mapper;
use uc_core::{clipboard::SnapshotRepresentation, ids::EventId};

pub struct SnapshotRepresentationRowMapper;

impl Mapper<(SnapshotRepresentation, EventId), NewSnapshotRepresentationRow>
    for SnapshotRepresentationRowMapper
{
    fn to_row(&self, domain: &(SnapshotRepresentation, EventId)) -> NewSnapshotRepresentationRow {
        let (rep, event_id) = domain;
        NewSnapshotRepresentationRow {
            id: rep.id.to_string(),
            event_id: event_id.to_string(),
            format_id: rep.format_id.to_string(),
            mime_type: rep.mime_type.clone(),
            size_bytes: rep.size_bytes,
            inline_data: rep.inline_data.clone(),
            blob_id: rep.blob_id.as_ref().map(|id| id.to_string()),
        }
    }
}
