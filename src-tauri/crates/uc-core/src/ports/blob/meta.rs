use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    /// MIME type of the original clipboard item
    pub mime: String,

    /// Additional metadata as key-value pairs
    pub meta: std::collections::BTreeMap<String, String>,
}
