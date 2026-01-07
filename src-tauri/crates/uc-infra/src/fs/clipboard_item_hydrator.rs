use anyhow::Result;

use uc_core::clipboard::ClipboardItem;
use uc_core::ports::BlobMeta;

use crate::codec;

/// Convert a `ClipboardItem` into a `BlobMeta` and its encoded blob bytes.
///
/// The returned `BlobMeta` contains the item's MIME type as a string and a clone of its metadata;
/// the second element is the result of encoding the item's data via the blob codec.
/// Returns an error if encoding fails.
///
/// # Examples
///
/// ```
/// // Construct a `ClipboardItem` appropriate for your project context:
/// let item = /* ClipboardItem */ unimplemented!();
/// let (meta, encoded) = dehydrate(&item).unwrap();
/// assert_eq!(meta.mime, item.mime.to_string());
/// ```
pub fn dehydrate(item: &ClipboardItem) -> Result<(BlobMeta, Vec<u8>)> {
    let blob_meta: BlobMeta = BlobMeta {
        mime: item.mime.to_string(),
        meta: item.meta.clone(),
    };
    let data = codec::blob::encode(&item.data)?;
    Ok((blob_meta, data))
}

/// Reconstructs a `ClipboardItem` from stored `BlobMeta` and raw bytes.
///
/// On success returns a `ClipboardItem` built from `meta` and `data`. If `meta.mime` begins with
/// "text/" the function decodes `data` as UTF-8 and returns a `ClipboardData::Text` containing the
/// decoded string; otherwise it returns a `ClipboardData::Bytes` containing the raw bytes.
/// Returns an error if MIME parsing fails or if text data is not valid UTF-8.
///
/// # Examples
///
/// ```
/// # use anyhow::Result;
/// # use uc_core::clipboard::ClipboardData;
/// # use uc_infra::fs::clipboard_item_hydrator::hydrate;
/// # use uc_infra::types::BlobMeta;
/// # use uc_core::clipboard::ClipboardItem;
/// # fn doc() -> Result<()> {
/// let meta = BlobMeta { mime: "text/plain".to_string(), meta: Default::default() };
/// let data = b"hello".to_vec();
/// let item = hydrate(meta, data)?;
/// match item.data {
///     ClipboardData::Text { text } => assert_eq!(text, "hello"),
///     _ => panic!("expected text"),
/// }
/// # Ok(()) }
/// ```
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