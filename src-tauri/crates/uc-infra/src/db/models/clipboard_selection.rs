#[derive(Queryable, Insertable)]
#[diesel(table_name = clipboard_selection)]
pub struct ClipboardSelectionRow {
    pub entry_id: String,
    pub primary_rep_id: String,
    pub preview_rep_id: String,
    pub paste_rep_id: String,
    pub policy_version: String,
}
