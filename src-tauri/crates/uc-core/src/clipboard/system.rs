/// 从系统剪切板中获取到原始数据的快照
#[derive(Debug, Clone)]
pub struct SystemClipboardSnapshot {
    pub ts_ms: i64,
    pub representations: Vec<SystemClipboardRepresentation>,
}

#[derive(Debug, Clone)]
pub struct SystemClipboardRepresentation {
    pub id: String, // 建议：uuid
    pub format_id: String,
    pub mime: Option<String>,
    pub size_bytes: i64,
    pub bytes: Vec<u8>,
}
