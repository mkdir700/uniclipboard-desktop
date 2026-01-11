use crate::{
    ids::{FormatId, RepresentationId},
    ContentHash, MimeType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotHash(pub ContentHash);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepresentationHash(pub ContentHash);

/// 从系统剪切板中获取到原始数据的快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemClipboardSnapshot {
    pub ts_ms: i64,
    pub representations: Vec<ObservedClipboardRepresentation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedClipboardRepresentation {
    pub id: RepresentationId, // 建议：uuid
    pub format_id: FormatId,
    pub mime: Option<MimeType>,
    pub bytes: Vec<u8>,
}

impl std::ops::Deref for RepresentationHash {
    type Target = ContentHash;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for SnapshotHash {
    type Target = ContentHash;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObservedClipboardRepresentation {
    pub fn size_bytes(&self) -> i64 {
        self.bytes.len() as i64
    }

    pub fn content_hash(&self) -> RepresentationHash {
        RepresentationHash(ContentHash::from(self.bytes.as_slice()))
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

    pub fn snapshot_hash(&self) -> SnapshotHash {
        let mut rep_hashes: Vec<[u8; 32]> = self
            .representations
            .iter()
            .map(|r| {
                let content_hash = r.content_hash();
                let hash_bytes = content_hash.as_ref();
                hash_bytes
                    .try_into()
                    .expect("ContentHash should be 32 bytes")
            })
            .collect();

        // 顺序无关
        rep_hashes.sort_unstable();

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"snapshot-hash-v1|");

        for h in &rep_hashes {
            hasher.update(h);
        }

        let hash = hasher.finalize();
        SnapshotHash(ContentHash::from(&hash.as_bytes()[..]))
    }
}
