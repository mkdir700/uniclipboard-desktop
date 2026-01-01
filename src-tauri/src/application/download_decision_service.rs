use crate::config::Setting;
use crate::domain::clipboard_metadata::ClipboardMetadata;
use crate::domain::content_type::ContentType;
use crate::domain::transfer_message::ClipboardTransferMessage;

/// 下载决策器
///
/// 负责决定是否下载远程剪贴板内容
#[derive(Clone)]
pub struct DownloadDecisionMaker {
    setting: Setting,
}

impl DownloadDecisionMaker {
    pub fn new() -> Self {
        // TODO: 需要创建一个全局缓存，当发生配置文件发生变大时，全局缓存可以统一更新，进而减少读文件的次数
        let setting = Setting::get_instance();
        Self { setting }
    }

    /// 决定是否下载内容
    pub async fn should_download(&self, message: &ClipboardTransferMessage) -> bool {
        let content_type = message.metadata.get_content_type();

        // 根据内容类型检查用户设置
        match content_type {
            ContentType::Text => self.setting.sync.content_types.text,
            ContentType::Image => self.setting.sync.content_types.image,
            ContentType::Link => self.setting.sync.content_types.link,
            ContentType::File => self.setting.sync.content_types.file,
            ContentType::CodeSnippet => self.setting.sync.content_types.code_snippet,
            ContentType::RichText => self.setting.sync.content_types.rich_text,
        }
    }

    /// 决定下载优先级
    ///
    /// 返回值越小优先级越高
    pub async fn get_download_priority(&self, message: &ClipboardTransferMessage) -> u8 {
        let content_type = message.metadata.get_content_type();

        // 根据内容类型设置基础优先级
        let base_priority = match content_type {
            ContentType::Text => 1, // 文本优先级最高
            ContentType::Link => 2, // 链接次之
            ContentType::CodeSnippet => 3,
            ContentType::RichText => 4,
            ContentType::Image => 5, // 图片优先级较低
            ContentType::File => 10, // 文件优先级最低
        };

        // 根据文件大小调整优先级
        // 大文件优先级降低
        let size_adjustment = if content_type == ContentType::Image {
            match &message.metadata {
                ClipboardMetadata::Image(img) => {
                    if img.size > 10 * 1024 * 1024 {
                        // 大于10MB
                        5
                    } else if img.size > 1 * 1024 * 1024 {
                        // 大于1MB
                        2
                    } else {
                        0
                    }
                }
                _ => 0,
            }
        } else {
            0
        };

        base_priority + size_adjustment
    }

    /// 检查是否超过最大允许大小
    pub fn exceeds_max_size(&self, message: &ClipboardTransferMessage) -> bool {
        let max_size_mb = self.setting.sync.max_file_size as u64;
        let max_size_bytes = max_size_mb * 1024 * 1024;

        match &message.metadata {
            ClipboardMetadata::Text(text) => (text.length as u64) > max_size_bytes,
            ClipboardMetadata::Image(img) => (img.size as u64) > max_size_bytes,
            ClipboardMetadata::Link(link) => (link.length as u64) > max_size_bytes,
            ClipboardMetadata::File(file) => (file.get_total_size() as u64) > max_size_bytes,
            ClipboardMetadata::CodeSnippet(code) => (code.length as u64) > max_size_bytes,
            ClipboardMetadata::RichText(rich_text) => (rich_text.length as u64) > max_size_bytes,
        }
    }
}
