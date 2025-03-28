use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// 剪贴板内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Link,
    File,
    CodeSnippet,
    RichText,
}

impl ContentType {
    /// 获取内容类型的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Link => "link",
            ContentType::File => "file",
            ContentType::CodeSnippet => "code_snippet",
            ContentType::RichText => "rich_text",
        }
    }

    /// 从字符串解析内容类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(ContentType::Text),
            "image" => Some(ContentType::Image),
            "link" => Some(ContentType::Link),
            "file" => Some(ContentType::File),
            "code_snippet" => Some(ContentType::CodeSnippet),
            "rich_text" => Some(ContentType::RichText),
            _ => None,
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
