#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardSelection {
    pub primary_rep_id: String,
    pub preview_rep_id: String,
    pub paste_rep_id: String,
    pub policy_version: String, // "v1"
}