use anyhow::Result;

use uc_core::clipboard::ClipboardItem;
use uc_core::ports::BlobMeta;

use crate::codec;

pub fn dehydrate(item: &ClipboardItem) -> Result<(BlobMeta, Vec<u8>)> {
    let blob_meta: BlobMeta = BlobMeta {
        mime: item.mime.to_string(),
        meta: item.meta.clone(),
    };
    let data = codec::blob::encode(&item.data)?;
    Ok((blob_meta, data))
}

pub fn hydrate(meta: BlobMeta, data: Vec<u8>) -> Result<ClipboardItem> {
    let mime = meta.mime.parse()?;
    // TODO: 这里是否需要优化，避免字符串硬编码
    match meta.mime.as_str() {
        m if m.starts_with("text/") => Ok(ClipboardItem {
            mime,
            meta: meta.meta,
            data: uc_core::clipboard::ClipboardData::Text {
                text: String::from_utf8(data)?,
            },
        }),
        _ => Ok(ClipboardItem {
            mime,
            meta: meta.meta,
            data: uc_core::clipboard::ClipboardData::Bytes { bytes: data },
        }),
    }
}
