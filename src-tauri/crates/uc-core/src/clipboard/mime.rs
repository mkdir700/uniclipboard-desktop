use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MimeType(pub String);

impl MimeType {
    pub fn text_plain() -> Self {
        Self("text/plain".into())
    }
    pub fn text_html() -> Self {
        Self("text/html".into())
    }
}
