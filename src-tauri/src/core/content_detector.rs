use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use twox_hash::xxh3::hash64;

use crate::core::clipboard_metadata::ClipboardMetadata;
use crate::core::content_type::ContentType;
use crate::core::metadata_models::TextMetadata;

// 使用 lazy_static 初始化正则表达式，避免每次调用方法时都重新编译
lazy_static! {
    /// URL 链接正则表达式
    static ref URL_REGEX: Regex = Regex::new(
        r"^(https?://)([-a-zA-Z0-9]+\.)+[a-zA-Z0-9]+([-a-zA-Z0-9@:%_\+.~#?&//=]*)$"
    ).unwrap();

    /// 文件路径正则表达式
    /// 支持 Unix/Linux 和 Windows 风格的文件路径
    static ref FILE_PATH_REGEX: Regex = Regex::new(
        r"^(/[a-zA-Z0-9_\-\.]+)+(/[a-zA-Z0-9_\-\.]+\.[a-zA-Z0-9]+)?$|^([A-Za-z]:)?\\([a-zA-Z0-9_\-\.]+\\)*[a-zA-Z0-9_\-\.]+(\.[a-zA-Z0-9]+)?$"
    ).unwrap();

    /// 代码片段正则表达式
    /// 通过识别常见的编程语言关键字和特征来判断内容是否为代码
    static ref CODE_SNIPPET_REGEX: Regex = Regex::new(
        r"(\{|\}|function\s+\w+\s*\(|class\s+\w+|public\s+|private\s+|def\s+\w+|fn\s+\w+|impl|struct|enum|for\s+.*\{|if\s+.*\{|while\s+.*\{|match\s+.*\{)"
    ).unwrap();

    /// 富文本内容正则表达式
    /// 检测HTML标签、Markdown格式等富文本特征
    static ref RICH_TEXT_REGEX: Regex = Regex::new(
        r"(<[a-zA-Z][^>]*>.*</[a-zA-Z][^>]*>|<[a-zA-Z][^>]*/>|\*\*.*\*\*|__.*__|#+\s+.*|>\s+.*|\[.*\]\(.*\)|\|.*\|.*\|)"
    ).unwrap();
}

/// 内容类型探测接口
///
/// 定义内容类型探测的公共接口，允许多种不同的探测器实现
pub trait ContentTypeDetector {
    /// 检测内容类型
    fn detect(&self, content: &[u8]) -> ContentType;
}

/// 正则表达式内容类型探测器
#[derive(Default)]
pub struct RegexContentDetector;

impl ContentTypeDetector for RegexContentDetector {
    fn detect(&self, content: &[u8]) -> ContentType {
        // 安全地将字节转换为字符串，如果转换失败则使用空字符串
        let content_str = match std::str::from_utf8(content) {
            Ok(s) => s.trim(),
            Err(_) => "",
        };

        // 无内容或无法转换为UTF-8时返回文本类型
        if content_str.is_empty() {
            return ContentType::Text;
        }

        // 按优先级检测内容类型
        if URL_REGEX.is_match(content_str) {
            ContentType::Link
        } else if FILE_PATH_REGEX.is_match(content_str) {
            ContentType::File
        } else if content_str.len() > 10 && CODE_SNIPPET_REGEX.is_match(content_str) {
            ContentType::CodeSnippet
        } else if RICH_TEXT_REGEX.is_match(content_str) {
            ContentType::RichText
        } else {
            ContentType::Text
        }
    }
}

/// 内容检测器
///
/// 使用策略模式允许不同类型的内容探测
#[derive(Default)]
pub struct ContentDetector {
    detector: RegexContentDetector,
}

impl ContentDetector {
    /// 创建新的内容检测器
    pub fn new() -> Self {
        Self {
            detector: RegexContentDetector::default(),
        }
    }

    /// 检测文本内容的类型
    pub fn detect_text_type(content: &[u8]) -> ContentType {
        let detector = Self::new();
        detector.detector.detect(content)
    }

    /// 为给定的文本内容创建适当类型的元数据
    pub fn create_text_metadata(
        content: &[u8],
        device_id: String,
        timestamp: DateTime<Utc>,
        storage_path: String,
    ) -> ClipboardMetadata {
        // 创建元数据基础结构
        let metadata = TextMetadata {
            content_hash: hash64(content),
            device_id,
            timestamp,
            length: content.len(),
            size: content.len(),
            storage_path,
        };

        // 识别内容类型并创建相应元数据
        match Self::detect_text_type(content) {
            ContentType::Link => ClipboardMetadata::Link(metadata),
            ContentType::CodeSnippet => ClipboardMetadata::CodeSnippet(metadata),
            ContentType::RichText => ClipboardMetadata::RichText(metadata),
            _ => ClipboardMetadata::Text(metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_link() {
        let url = b"https://www.example.com/page.html";
        assert_eq!(ContentDetector::detect_text_type(url), ContentType::Link);

        let url_with_params = b"https://example.com/search?q=rust&lang=en";
        assert_eq!(
            ContentDetector::detect_text_type(url_with_params),
            ContentType::Link
        );
    }

    #[test]
    fn test_detect_file_path() {
        let unix_path = b"/home/user/documents/file.txt";
        assert_eq!(
            ContentDetector::detect_text_type(unix_path),
            ContentType::File
        );

        let windows_path = b"C:\\Users\\user\\Documents\\file.txt";
        assert_eq!(
            ContentDetector::detect_text_type(windows_path),
            ContentType::File
        );
    }

    #[test]
    fn test_detect_code_snippet() {
        let rust_code = b"fn main() {\n    println!(\"Hello, World!\");\n}";
        assert_eq!(
            ContentDetector::detect_text_type(rust_code),
            ContentType::CodeSnippet
        );

        let js_code = b"function greet(name) {\n    return `Hello, ${name}`;\n}";
        assert_eq!(
            ContentDetector::detect_text_type(js_code),
            ContentType::CodeSnippet
        );
    }

    #[test]
    fn test_detect_rich_text() {
        let html = b"<div><h1>Title</h1><p>Content</p></div>";
        assert_eq!(
            ContentDetector::detect_text_type(html),
            ContentType::RichText
        );

        let markdown = b"# Heading\n\n**Bold text** with [link](https://example.com)";
        assert_eq!(
            ContentDetector::detect_text_type(markdown),
            ContentType::RichText
        );
    }

    #[test]
    fn test_detect_plain_text() {
        let text = b"This is just a plain text without any special format.";
        assert_eq!(ContentDetector::detect_text_type(text), ContentType::Text);
    }

    #[test]
    fn test_invalid_utf8() {
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        assert_eq!(
            ContentDetector::detect_text_type(invalid_bytes),
            ContentType::Text
        );
    }

    #[test]
    fn test_empty_content() {
        let empty = b"";
        assert_eq!(ContentDetector::detect_text_type(empty), ContentType::Text);
    }
}
