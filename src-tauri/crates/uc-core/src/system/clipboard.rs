use crate::clipboard::MimeType;

/// 从系统剪切板中获取到原始数据的快照
#[derive(Debug, Clone)]
pub struct RawClipboardSnapshot {
    pub ts_ms: i64,
    pub representations: Vec<RawClipboardRepresentation>,
}

#[derive(Debug, Clone)]
pub struct RawClipboardRepresentation {
    // 格式标识符（字符串） 来源系统剪切板的格式标识符
    pub format_id: String,
    pub mime: Option<MimeType>,
    pub bytes: Vec<u8>,
}
