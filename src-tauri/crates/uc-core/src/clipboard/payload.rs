//! Clipboard payload domain model
//!
//! Represents clipboard content that can be text, image, or files.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twox_hash::xxh3::hash64;

/// Clipboard payload enum representing different content types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Payload {
    Text(TextPayload),
    Image(ImagePayload),
    File(FilePayload),
}

/// Text clipboard payload
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TextPayload {
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    content: Bytes,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

impl TextPayload {
    pub fn new(content: Bytes, device_id: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            content,
            device_id,
            timestamp,
        }
    }

    pub fn get_content(&self) -> Bytes {
        self.content.clone()
    }
}

/// Image clipboard payload
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ImagePayload {
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    content: Bytes,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub size: usize,
}

impl ImagePayload {
    pub fn new(
        content: Bytes,
        device_id: String,
        timestamp: DateTime<Utc>,
        width: usize,
        height: usize,
        format: String,
        size: usize,
    ) -> Self {
        Self {
            content,
            device_id,
            timestamp,
            width,
            height,
            format,
            size,
        }
    }

    pub fn get_content(&self) -> Bytes {
        self.content.clone()
    }

    /// Compute content hash for deduplication (sample hash: only hash first 64KB)
    pub fn content_hash(&self) -> u64 {
        const SAMPLE_SIZE: usize = 64 * 1024; // 64KB
        let sample = if self.content.len() <= SAMPLE_SIZE {
            self.content.as_ref()
        } else {
            &self.content[..SAMPLE_SIZE]
        };
        hash64(sample)
    }
}

/// File information for file payloads
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub file_path: String,
}

/// File clipboard payload
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FilePayload {
    pub content_hash: u64,
    pub file_infos: Vec<FileInfo>,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

impl FilePayload {
    pub fn new(file_infos: Vec<FileInfo>, device_id: String, timestamp: DateTime<Utc>) -> Self {
        // Hash all file paths concatenated
        let paths_bytes: Vec<u8> = file_infos
            .iter()
            .flat_map(|f| f.file_path.as_bytes())
            .copied()
            .collect();
        let content_hash = hash64(&paths_bytes);
        Self {
            content_hash,
            file_infos,
            device_id,
            timestamp,
        }
    }

    /// Get file paths from the payload
    pub fn get_file_paths(&self) -> Vec<String> {
        self.file_infos
            .iter()
            .map(|f| f.file_path.clone())
            .collect()
    }
}

/// Helper to serialize bytes as base64
fn serialize_bytes<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use base64::Engine;
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&base64_string)
}

/// Helper to deserialize bytes from base64
fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::Engine;
    let base64_string = String::deserialize(deserializer)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_string)
        .map_err(|e: base64::DecodeError| serde::de::Error::custom(e.to_string()))?;
    Ok(Bytes::from(bytes))
}

impl Payload {
    /// Create a new text payload
    pub fn new_text(content: Bytes, device_id: String, timestamp: DateTime<Utc>) -> Self {
        Payload::Text(TextPayload::new(content, device_id, timestamp))
    }

    /// Create a new image payload
    pub fn new_image(
        content: Bytes,
        device_id: String,
        timestamp: DateTime<Utc>,
        width: usize,
        height: usize,
        format: String,
        size: usize,
    ) -> Self {
        Payload::Image(ImagePayload::new(
            content, device_id, timestamp, width, height, format, size,
        ))
    }

    /// Create a new file payload
    pub fn new_file(
        file_infos: Vec<FileInfo>,
        device_id: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Payload::File(FilePayload::new(file_infos, device_id, timestamp))
    }

    /// Get the timestamp of the payload
    pub fn get_timestamp(&self) -> DateTime<Utc> {
        match self {
            Payload::Text(p) => p.timestamp,
            Payload::Image(p) => p.timestamp,
            Payload::File(p) => p.timestamp,
        }
    }

    /// Check if this is an image payload
    pub fn is_image(&self) -> bool {
        matches!(self, Payload::Image(_))
    }

    /// Check if this is a text payload
    pub fn is_text(&self) -> bool {
        matches!(self, Payload::Text(_))
    }

    /// Get image payload reference if this is an image
    pub fn as_image(&self) -> Option<&ImagePayload> {
        match self {
            Payload::Image(img) => Some(img),
            _ => None,
        }
    }

    /// Get the device ID that created this payload
    pub fn get_device_id(&self) -> &str {
        match self {
            Payload::Text(p) => &p.device_id,
            Payload::Image(p) => &p.device_id,
            Payload::File(p) => &p.device_id,
        }
    }

    /// Get the content type as a string
    pub fn content_type(&self) -> &str {
        match self {
            Payload::Text(_) => "text",
            Payload::Image(_) => "image",
            Payload::File(_) => "file",
        }
    }

    /// Get unique key for this payload
    pub fn get_key(&self) -> String {
        match self {
            Payload::Text(p) => {
                format!("{:016x}", hash64(p.content.as_ref()))
            }
            Payload::Image(p) => {
                let content_hash = p.content_hash();
                let size_info = format!("{}x{}", p.width, p.height);
                format!("img_{:016x}_{}", content_hash, size_info)
            }
            Payload::File(p) => {
                format!("file_{:016x}", p.content_hash)
            }
        }
    }

    /// Check if two payloads are duplicates
    pub fn is_duplicate(&self, other: &Payload) -> bool {
        match (self, other) {
            (Payload::Text(t1), Payload::Text(t2)) => t1.content == t2.content,
            (Payload::Image(i1), Payload::Image(i2)) => {
                // Special handling for size=0 to avoid division by zero
                let size_match = if i1.size == 0 && i2.size == 0 {
                    true
                } else if i1.size == 0 || i2.size == 0 {
                    false
                } else {
                    (i1.size as f64 - i2.size as f64).abs() / (i1.size as f64) <= 0.1
                };

                i1.content_hash() == i2.content_hash()
                    && i1.width == i2.width
                    && i1.height == i2.height
                    && size_match
            }
            _ => false,
        }
    }

    /// Get the content hash for deduplication
    pub fn content_hash(&self) -> String {
        match self {
            Payload::Text(p) => format!("{:016x}", hash64(p.content.as_ref())),
            Payload::Image(p) => format!("{:016x}", p.content_hash()),
            Payload::File(p) => format!("{:016x}", p.content_hash),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_payload_creation() {
        let content = Bytes::from("Hello, World!");
        let payload = Payload::new_text(content.clone(), "device123".to_string(), Utc::now());

        assert!(payload.is_text());
        assert_eq!(payload.get_device_id(), "device123");
    }

    #[test]
    fn test_image_payload_hash() {
        let content = Bytes::from(vec![0xAB; 32 * 1024]); // 32KB
        let img = ImagePayload::new(
            content.clone(),
            "device123".to_string(),
            Utc::now(),
            1920,
            1080,
            "png".to_string(),
            32768,
        );

        let hash1 = img.content_hash();
        let hash2 = img.content_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_payload_is_duplicate() {
        let content = Bytes::from("test content");
        let payload1 = Payload::new_text(content.clone(), "device123".to_string(), Utc::now());
        let payload2 = Payload::new_text(content, "device456".to_string(), Utc::now());

        assert!(payload1.is_duplicate(&payload2));
    }
}
