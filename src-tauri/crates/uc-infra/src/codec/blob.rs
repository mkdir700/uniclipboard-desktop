use anyhow::Result;
use uc_core::clipboard::ClipboardData;

/// Encodes a ClipboardData value into a byte vector.
///
/// Text variants are encoded as UTF-8 bytes of the contained string. Byte variants are encoded
/// as a clone of the contained bytes.
///
/// # Returns
///
/// A `Vec<u8>` containing the encoded representation of `data`.
///
/// # Examples
///
/// ```
/// use uc_core::clipboard::ClipboardData;
/// // assume `encode` is in scope
/// let txt = ClipboardData::Text { text: "hello".into() };
/// let out = encode(&txt).unwrap();
/// assert_eq!(out, b"hello");
/// ```
pub fn encode(data: &ClipboardData) -> Result<Vec<u8>> {
    match data {
        ClipboardData::Text { text } => Ok(text.as_bytes().to_vec()),
        ClipboardData::Bytes { bytes } => Ok(bytes.clone()),
    }
}

/// Decode a byte buffer into `ClipboardData` based on the provided MIME type.
///
/// If `mime` starts with `"text/"`, the bytes are interpreted as UTF-8 and returned as
/// `ClipboardData::Text`. For other MIME types the original bytes are returned as
/// `ClipboardData::Bytes`.
///
/// # Errors
///
/// Returns an error if `mime` starts with `"text/"` but `bytes` are not valid UTF-8.
///
/// # Examples
///
/// ```
/// use uc_core::clipboard::ClipboardData;
/// let txt = decode(b"hello".to_vec(), "text/plain").unwrap();
/// match txt {
///     ClipboardData::Text { text } => assert_eq!(text, "hello"),
///     _ => panic!("expected text"),
/// }
///
/// let bin = decode(vec![0, 159, 146, 150], "application/octet-stream").unwrap();
/// match bin {
///     ClipboardData::Bytes { bytes } => assert_eq!(bytes, vec![0, 159, 146, 150]),
///     _ => panic!("expected bytes"),
/// }
/// ```
pub fn decode(bytes: Vec<u8>, mime: &str) -> Result<ClipboardData> {
    if mime.starts_with("text/") {
        Ok(ClipboardData::Text {
            text: String::from_utf8(bytes)?,
        })
    } else {
        Ok(ClipboardData::Bytes { bytes })
    }
}