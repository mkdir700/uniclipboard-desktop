/// 选择目标：UI 预览 vs 默认粘贴
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionTarget {
    UiPreview,
    DefaultPaste,
}

/// 选择结果（建议落库：clipboard_selection）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardSelection {
    pub primary_rep_id: String,
    pub preview_rep_id: String,
    pub paste_rep_id: String,
    pub policy_version: String, // "v1"
}