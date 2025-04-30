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

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Link => "link",
            ContentType::File => "file",
            ContentType::CodeSnippet => "code",
            ContentType::RichText => "rich_text",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for ContentType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "text" => Ok(ContentType::Text),
            "image" => Ok(ContentType::Image),
            "link" => Ok(ContentType::Link),
            "file" => Ok(ContentType::File),
            "code" => Ok(ContentType::CodeSnippet),
            "rich_text" => Ok(ContentType::RichText),
            _ => Err(format!("无效的内容类型: {}", s)),
        }
    }
}

// 将 ContentType 转换为 String
impl From<ContentType> for String {
    fn from(content_type: ContentType) -> Self {
        match content_type {
            ContentType::Text => "text".to_string(),
            ContentType::Image => "image".to_string(),
            ContentType::Link => "link".to_string(),
            ContentType::File => "file".to_string(),
            ContentType::CodeSnippet => "code".to_string(),
            ContentType::RichText => "rich_text".to_string(),
        }
    }
}

// 为 &String 实现 TryFrom
impl TryFrom<&String> for ContentType {
    type Error = String;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}
