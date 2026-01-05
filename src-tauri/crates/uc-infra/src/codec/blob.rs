use anyhow::Result;
use uc_core::clipboard::ClipboardData;

pub fn encode(data: &ClipboardData) -> Result<Vec<u8>> {
    match data {
        ClipboardData::Text { text } => Ok(text.as_bytes().to_vec()),
        ClipboardData::Bytes { bytes } => Ok(bytes.clone()),
    }
}

pub fn decode(bytes: Vec<u8>, mime: &str) -> Result<ClipboardData> {
    if mime.starts_with("text/") {
        Ok(ClipboardData::Text {
            text: String::from_utf8(bytes)?,
        })
    } else {
        Ok(ClipboardData::Bytes { bytes })
    }
}
