//! Data Transfer Objects and Projection Models
//!
//! This module contains data structures that are exposed to the frontend.
//! These separate the internal domain models from the API contract.
//!
//! 数据传输对象和投影模型
//!
//! 此模块包含暴露给前端的数据结构。
//! 这些将内部领域模型与 API 契约分离。

use serde::{Deserialize, Serialize};

/// Clipboard entry projection for frontend API.
/// 前端 API 的剪贴板条目投影。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntryProjection {
    /// Unique identifier for the entry
    pub id: String,
    /// Preview text for display
    pub preview: String,
    /// Timestamp when captured (Unix timestamp)
    pub captured_at: i64,
    /// Content type description
    pub content_type: String,
    /// Whether the content is encrypted
    pub is_encrypted: bool,
    /// Whether the entry is favorited
    pub is_favorited: bool,
    /// Timestamp when last updated
    pub updated_at: i64,
    /// Timestamp of last access/use
    pub active_time: i64,
}
