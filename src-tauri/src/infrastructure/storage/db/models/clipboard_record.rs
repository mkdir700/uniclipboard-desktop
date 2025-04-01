use crate::core::{content_type::ContentType, ClipboardMetadata, ClipboardTransferMessage};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DbClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub local_file_path: Option<String>,
    pub remote_record_id: Option<String>,
    pub content_type: String,
    pub content_hash: Option<String>,
    pub content_size: Option<i32>,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub active_time: i32,
    pub extra: Option<String>,
}

impl DbClipboardRecord {
    pub fn new(
        id: String,
        device_id: String,
        local_file_path: Option<String>,
        remote_record_id: Option<String>,
        content_type: String,
        content_hash: Option<String>,
        content_size: Option<i32>,
        is_favorited: bool,
        created_at: i32,
        updated_at: i32,
        active_time: i32,
        extra: Option<ExtraInfo>,
    ) -> Result<Self, serde_json::Error> {
        let extra = if let Some(extra_info) = extra {
            Some(String::try_from(extra_info)?)
        } else {
            None
        };
        Ok(Self {
            id,
            device_id,
            local_file_path,
            remote_record_id,
            content_type,
            content_hash,
            content_size,
            is_favorited,
            created_at,
            updated_at,
            active_time,
            extra,
        })
    }

    /// 获取内容类型枚举
    pub fn get_content_type(&self) -> Option<ContentType> {
        ContentType::try_from(&self.content_type).ok()
    }

    /// 获取更新记录
    pub fn get_update_record(&self) -> UpdateClipboardRecord {
        UpdateClipboardRecord {
            is_favorited: self.is_favorited,
            updated_at: self.updated_at,
            active_time: self.active_time,
        }
    }

    /// 获取额外信息，支持嵌套和非嵌套格式
    pub fn get_typed_extra_enhanced(&self) -> Option<ExtraInfo> {
        let content_type = self.get_content_type()?;
        let extra_str = self.extra.as_ref()?;

        // 首先尝试解析为嵌套格式
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(extra_str) {
            // 检查是否有嵌套的JSON结构
            match content_type {
                ContentType::Text => {
                    if let Some(text_obj) = value.get("Text") {
                        return serde_json::from_value::<TextExtra>(text_obj.clone())
                            .ok()
                            .map(ExtraInfo::Text);
                    }
                }
                ContentType::Image => {
                    if let Some(img_obj) = value.get("Image") {
                        return serde_json::from_value::<ImageExtra>(img_obj.clone())
                            .ok()
                            .map(ExtraInfo::Image);
                    }
                }
                ContentType::File => {
                    if let Some(file_obj) = value.get("File") {
                        return serde_json::from_value::<FileExtra>(file_obj.clone())
                            .ok()
                            .map(ExtraInfo::File);
                    }
                }
                ContentType::RichText => {
                    if let Some(rt_obj) = value.get("RichText") {
                        return serde_json::from_value::<RichTextExtra>(rt_obj.clone())
                            .ok()
                            .map(ExtraInfo::RichText);
                    }
                }
                _ => {}
            }
        }

        // 如果嵌套格式解析失败，回退到标准格式
        self.get_typed_extra()
    }

    /// 根据内容类型获取对应的额外信息
    pub fn get_typed_extra(&self) -> Option<ExtraInfo> {
        let content_type = self.get_content_type()?;
        let extra = self.extra.as_ref()?;

        match content_type {
            ContentType::Text => serde_json::from_str::<TextExtra>(extra)
                .ok()
                .map(ExtraInfo::Text),
            ContentType::Image => serde_json::from_str::<ImageExtra>(extra)
                .ok()
                .map(ExtraInfo::Image),
            ContentType::File => serde_json::from_str::<FileExtra>(extra)
                .ok()
                .map(ExtraInfo::File),
            ContentType::RichText => serde_json::from_str::<RichTextExtra>(extra)
                .ok()
                .map(ExtraInfo::RichText),
            _ => None,
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
pub struct NewClipboardRecord {
    pub id: String,
    pub device_id: String,
    pub local_file_path: Option<String>,
    pub remote_record_id: Option<String>,
    pub content_type: String,
    pub content_hash: String,
    pub content_size: Option<i32>,
    pub is_favorited: bool,
    pub created_at: i32,
    pub updated_at: i32,
    pub extra: Option<String>,
}

impl NewClipboardRecord {
    /// 设置额外信息
    pub fn set_extra<T: Serialize>(&mut self, extra: &T) -> Result<(), serde_json::Error> {
        let json = serde_json::to_string(extra)?;
        self.extra = Some(json);
        Ok(())
    }

    /// 设置类型化的额外信息
    pub fn set_typed_extra(&mut self, extra: &ExtraInfo) -> Result<(), serde_json::Error> {
        self.set_extra(extra)
    }
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::infrastructure::storage::db::schema::clipboard_records)]
pub struct UpdateClipboardRecord {
    pub is_favorited: bool,
    pub updated_at: i32,
    pub active_time: i32,
}

// 为不同的内容类型定义额外信息结构体

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyExtra {}

/// 文本内容额外信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TextExtra {
    pub is_rich_text: Option<bool>,
}

/// 文件额外信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FileExtra {
    pub file_names: Vec<String>,
    pub file_sizes: Vec<usize>,
}

/// 图片额外信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageExtra {
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// 富文本额外信息
#[derive(Debug, Serialize, Deserialize)]
pub struct RichTextExtra {
    pub has_images: Option<bool>,
    pub html_version: Option<String>,
}

/// 额外信息枚举，包含所有可能的额外信息类型
#[derive(Debug, Serialize, Deserialize)]
pub enum ExtraInfo {
    Text(TextExtra),
    Image(ImageExtra),
    File(FileExtra),
    RichText(RichTextExtra),
    Empty(EmptyExtra),
}

impl TryFrom<ExtraInfo> for String {
    type Error = serde_json::Error;

    fn try_from(extra: ExtraInfo) -> Result<Self, Self::Error> {
        serde_json::to_string(&extra)
    }
}

impl TryFrom<&ClipboardMetadata> for ExtraInfo {
    type Error = serde_json::Error;

    fn try_from(metadata: &ClipboardMetadata) -> Result<Self, Self::Error> {
        match metadata {
            ClipboardMetadata::Image(image) => Ok(ExtraInfo::Image(ImageExtra {
                width: Some(image.width as i32),
                height: Some(image.height as i32),
            })),
            ClipboardMetadata::File(file) => Ok(ExtraInfo::File(FileExtra {
                file_names: file.file_names.clone(),
                file_sizes: file.file_sizes.clone(),
            })),
            _ => Ok(ExtraInfo::Empty(EmptyExtra {})),
        }
    }
}

impl ExtraInfo {
    /// 获取文本类型的额外信息
    pub fn as_text(&self) -> Option<&TextExtra> {
        if let ExtraInfo::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// 获取图片类型的额外信息
    pub fn as_image(&self) -> Option<&ImageExtra> {
        if let ExtraInfo::Image(image) = self {
            Some(image)
        } else {
            None
        }
    }

    /// 获取文件类型的额外信息
    pub fn as_file(&self) -> Option<&FileExtra> {
        if let ExtraInfo::File(file) = self {
            Some(file)
        } else {
            None
        }
    }

    /// 获取富文本类型的额外信息
    pub fn as_rich_text(&self) -> Option<&RichTextExtra> {
        if let ExtraInfo::RichText(rich_text) = self {
            Some(rich_text)
        } else {
            None
        }
    }

    /// 获取额外信息对应的内容类型
    pub fn get_content_type(&self) -> ContentType {
        match self {
            ExtraInfo::Text(_) => ContentType::Text,
            ExtraInfo::Image(_) => ContentType::Image,
            ExtraInfo::File(_) => ContentType::File,
            ExtraInfo::RichText(_) => ContentType::RichText,
            ExtraInfo::Empty(_) => ContentType::Text,
        }
    }
}

// 从ExtraInfo到String的转换
impl TryFrom<&ExtraInfo> for String {
    type Error = serde_json::Error;

    fn try_from(extra: &ExtraInfo) -> Result<Self, Self::Error> {
        serde_json::to_string(extra)
    }
}

/// 获取特定内容类型对应的额外信息类型
pub fn get_extra_type_for_content_type(content_type: ContentType) -> &'static str {
    match content_type {
        ContentType::Text => "TextExtra",
        ContentType::Image => "ImageExtra",
        ContentType::Link => "LinkExtra",
        ContentType::File => "FileExtra",
        ContentType::CodeSnippet => "CodeSnippetExtra",
        ContentType::RichText => "RichTextExtra",
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderBy {
    #[serde(rename = "created_at_asc")]
    CreatedAtAsc,
    #[serde(rename = "created_at_desc")]
    CreatedAtDesc,
    #[serde(rename = "updated_at_asc")]
    UpdatedAtAsc,
    #[serde(rename = "updated_at_desc")]
    UpdatedAtDesc,
    #[serde(rename = "active_time_asc")]
    ActiveTimeAsc,
    #[serde(rename = "active_time_desc")]
    ActiveTimeDesc,
    #[serde(rename = "content_type_asc")]
    ContentTypeAsc,
    #[serde(rename = "content_type_desc")]
    ContentTypeDesc,
    #[serde(rename = "is_favorited_asc")]
    IsFavoritedAsc,
    #[serde(rename = "is_favorited_desc")]
    IsFavoritedDesc,
}

impl Default for OrderBy {
    fn default() -> Self {
        OrderBy::ActiveTimeDesc
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Filter {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "favorited")]
    Favorited,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "file")]
    File,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

/// 为特定内容类型创建默认的额外信息
pub fn create_default_extra(content_type: ContentType) -> ExtraInfo {
    match content_type {
        ContentType::Text => ExtraInfo::Text(TextExtra {
            is_rich_text: Some(false),
        }),
        ContentType::Image => ExtraInfo::Image(ImageExtra {
            width: None,
            height: None,
        }),
        ContentType::File => ExtraInfo::File(FileExtra {
            file_names: vec![],
            file_sizes: vec![],
        }),
        ContentType::RichText => ExtraInfo::RichText(RichTextExtra {
            has_images: None,
            html_version: None,
        }),
        _ => ExtraInfo::Empty(EmptyExtra {}),
    }
}

/// 尝试从字符串解析特定类型的额外信息
pub fn parse_extra_for_content_type(
    content_type: ContentType,
    extra_str: &str,
) -> Option<ExtraInfo> {
    match content_type {
        ContentType::Text | ContentType::CodeSnippet | ContentType::Link => {
            serde_json::from_str::<TextExtra>(extra_str)
                .ok()
                .map(ExtraInfo::Text)
        }
        ContentType::Image => serde_json::from_str::<ImageExtra>(extra_str)
            .ok()
            .map(ExtraInfo::Image),
        ContentType::File => serde_json::from_str::<FileExtra>(extra_str)
            .ok()
            .map(ExtraInfo::File),
        ContentType::RichText => serde_json::from_str::<RichTextExtra>(extra_str)
            .ok()
            .map(ExtraInfo::RichText),
    }
}

impl TryFrom<&ClipboardMetadata> for Option<ExtraInfo> {
    type Error = serde_json::Error;

    fn try_from(metadata: &ClipboardMetadata) -> Result<Self, Self::Error> {
        Ok(Some(ExtraInfo::try_from(metadata)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_typed_extra() {
        // 测试直接格式的JSON（直接存储FileExtra结构）
        let mut record = DbClipboardRecord {
            id: "test".to_string(),
            device_id: "device".to_string(),
            local_file_path: Some("path".to_string()),
            remote_record_id: None,
            content_type: "file".to_string(),
            content_hash: Some("hash".to_string()),
            content_size: Some(100),
            is_favorited: false,
            created_at: 0,
            updated_at: 0,
            active_time: 0,
            extra: Some(r#"{"file_names":["test.txt"],"file_sizes":[1024]}"#.to_string()),
        };

        // 测试直接格式
        let extra = record.get_typed_extra();
        assert!(extra.is_some());
        let extra = extra.unwrap();
        if let ExtraInfo::File(file) = extra {
            assert_eq!(file.file_names, vec!["test.txt"]);
            assert_eq!(file.file_sizes, vec![1024]);
        } else {
            panic!("Expected File extra info!");
        }

        // 测试嵌套格式的JSON（带有类型标记的嵌套结构）
        record.extra =
            Some(r#"{"File":{"file_names":["crawl_details.py"],"file_sizes":[1074]}}"#.to_string());

        // 当前实现无法直接处理嵌套格式，这个测试应该失败
        let extra = record.get_typed_extra();
        assert!(extra.is_none(), "当前实现应无法处理嵌套格式");
    }

    #[test]
    fn test_get_typed_extra_enhanced() {
        // 创建测试记录
        let mut record = DbClipboardRecord {
            id: "test".to_string(),
            device_id: "device".to_string(),
            local_file_path: Some("path".to_string()),
            remote_record_id: None,
            content_type: "file".to_string(),
            content_hash: Some("hash".to_string()),
            content_size: Some(100),
            is_favorited: false,
            created_at: 0,
            updated_at: 0,
            active_time: 0,
            extra: None,
        };

        // 测试标准格式
        record.extra = Some(r#"{"file_names":["test.txt"],"file_sizes":[1024]}"#.to_string());
        let extra = record.get_typed_extra_enhanced();
        assert!(extra.is_some());
        if let Some(ExtraInfo::File(file)) = extra {
            assert_eq!(file.file_names, vec!["test.txt"]);
            assert_eq!(file.file_sizes, vec![1024]);
        } else {
            panic!("Expected File extra info!");
        }

        // 测试嵌套格式
        record.extra =
            Some(r#"{"File":{"file_names":["crawl_details.py"],"file_sizes":[1074]}}"#.to_string());
        let extra = record.get_typed_extra_enhanced();
        assert!(extra.is_some(), "增强方法应该能处理嵌套格式");
        if let Some(ExtraInfo::File(file)) = extra {
            assert_eq!(file.file_names, vec!["crawl_details.py"]);
            assert_eq!(file.file_sizes, vec![1074]);
        } else {
            panic!("Expected File extra info from nested format!");
        }

        // 测试图片类型的嵌套格式
        record.content_type = "image".to_string();
        record.extra = Some(r#"{"Image":{"width":800,"height":600}}"#.to_string());
        let extra = record.get_typed_extra_enhanced();
        assert!(extra.is_some());
        if let Some(ExtraInfo::Image(img)) = extra {
            assert_eq!(img.width, Some(800));
            assert_eq!(img.height, Some(600));
        } else {
            panic!("Expected Image extra info from nested format!");
        }
    }

    #[test]
    fn test_parse_nested_json() {
        // 测试用于解析嵌套JSON的辅助函数
        let nested_json = r#"{"File":{"file_names":["crawl_details.py"],"file_sizes":[1074]}}"#;
        let parsed: serde_json::Value = serde_json::from_str(nested_json).unwrap();

        // 尝试提取内部结构
        if let Some(file_obj) = parsed.get("File") {
            let file_extra: Result<FileExtra, _> = serde_json::from_value(file_obj.clone());
            assert!(file_extra.is_ok());
            let file_extra = file_extra.unwrap();
            assert_eq!(file_extra.file_names, vec!["crawl_details.py"]);
            assert_eq!(file_extra.file_sizes, vec![1074]);
        } else {
            panic!("Expected 'File' key in JSON!");
        }
    }
}
