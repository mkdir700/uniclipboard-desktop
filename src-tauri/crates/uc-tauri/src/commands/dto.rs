use serde::{Deserialize, Serialize};

/// Clipboard item DTO for frontend API.
///
/// This DTO separates the frontend API from internal domain models,
/// allowing domain evolution without breaking the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemDto {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub content_type: String,
    pub preview: Option<String>,
}

// Conversion implementations will be added after checking uc-core models
