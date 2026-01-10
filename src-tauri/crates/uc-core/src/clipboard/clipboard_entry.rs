pub struct NewClipboardEntry {
    pub entry_id: String,
    pub event_id: String,
    pub created_at_ms: i64,
    pub title: Option<String>,
    pub total_size: i64,
}

pub struct NewClipboardSelection {
    pub entry_id: String,
    pub primary_rep_id: String,
    pub preview_rep_id: String,
    pub paste_rep_id: String,
    pub policy_version: String,
}

