use crate::db::models::clipboard_selection::NewClipboardSelectionRow;
use crate::db::ports::InsertMapper;
use anyhow::Result;
use uc_core::clipboard::ClipboardSelectionDecision;

pub struct ClipboardSelectionRowMapper;

impl InsertMapper<ClipboardSelectionDecision, NewClipboardSelectionRow>
    for ClipboardSelectionRowMapper
{
    fn to_row(&self, domain: &ClipboardSelectionDecision) -> Result<NewClipboardSelectionRow> {
        let secondary_rep_ids = domain
            .selection
            .secondary_rep_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Ok(NewClipboardSelectionRow {
            entry_id: domain.entry_id.to_string(),
            primary_rep_id: domain.selection.primary_rep_id.to_string(),
            secondary_rep_ids,
            preview_rep_id: domain.selection.preview_rep_id.to_string(),
            paste_rep_id: domain.selection.paste_rep_id.to_string(),
            policy_version: domain.selection.policy_version.to_string(),
        })
    }
}
