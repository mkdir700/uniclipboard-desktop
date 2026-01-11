use crate::{
    ids::{FormatId, RepresentationId},
    MimeType,
};

/// 从系统剪切板中获取到原始数据的快照
#[derive(Debug, Clone)]
pub struct SystemClipboardSnapshot {
    pub ts_ms: i64,
    pub representations: Vec<ObservedClipboardRepresentation>,
}

#[derive(Debug, Clone)]
pub struct ObservedClipboardRepresentation {
    pub id: RepresentationId, // 建议：uuid
    pub format_id: FormatId,
    pub mime: Option<MimeType>,
    pub bytes: Vec<u8>,
}

impl ObservedClipboardRepresentation {
    pub fn size_bytes(&self) -> i64 {
        self.bytes.len() as i64
    }
}

impl SystemClipboardSnapshot {
    /// 返回该快照中所有 representation 的总字节大小
    pub fn total_size_bytes(&self) -> i64 {
        self.representations.iter().map(|r| r.size_bytes()).sum()
    }

    /// 是否为空快照（没有任何 representation）
    pub fn is_empty(&self) -> bool {
        self.representations.is_empty()
    }

    /// representation 数量
    pub fn representation_count(&self) -> usize {
        self.representations.len()
    }
}
