use crate::db::models::clipboard_selection::NewClipboardSelectionRow;
use crate::db::ports::InsertMapper;
use uc_core::clipboard::ClipboardSelectionDecision;
use anyhow::Result;

pub struct ClipboardSelectionRowMapper;

impl InsertMapper<ClipboardSelectionDecision, NewClipboardSelectionRow> for ClipboardSelectionRowMapper {
    fn to_row(&self, domain: &ClipboardSelectionDecision) -> Result<NewClipboardSelectionRow> {
        Ok(NewClipboardSelectionRow {
            entry_id: domain.entry_id.clone().to_string(),
            primary_rep_id: domain.selection.primary_rep_id.clone().to_string(),
            preview_rep_id: domain.selection.preview_rep_id.clone().to_string(),
            paste_rep_id: domain.selection.paste_rep_id.clone().to_string(),
            policy_version: domain.selection.policy_version.clone().to_string(),
        })
    }
}
