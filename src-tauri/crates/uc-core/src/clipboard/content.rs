//! Clipboard content model and responsibility boundaries
//!
//! This module defines the core data structures used to represent clipboard
//! data in a platform-agnostic and transport-friendly form.
//!
//! ## Design overview
//!
//! Clipboard data is modeled at two distinct but related levels:
//!
//! - [`ClipboardContent`] represents a **single clipboard snapshot**,
//!   corresponding to one copy/cut action observed at a specific point in time.
//! - [`ClipboardItem`] represents **one concrete data representation**
//!   (e.g. `text/plain`, `image/png`) that belongs to the same snapshot.
//!
//! This separation is intentional and fundamental to the design.
//!
//! ## ClipboardContent (snapshot level)
//!
//! [`ClipboardContent`] is the atomic unit of clipboard history, deduplication,
//! storage, and synchronization.
//!
//! A single clipboard snapshot may contain multiple representations of the same
//! logical content (for example plain text, HTML, and rich text), all grouped
//! under one [`ClipboardContent`] instance.
//!
//! Responsibilities of [`ClipboardContent`] include:
//! - Identifying **when** a clipboard change occurred (`ts_ms`)
//! - Grouping all representations that belong to the **same copy operation**
//! - Acting as the unit of deduplication and synchronization
//! - Holding snapshot-level metadata (e.g. source hints, debug attributes)
//!
//! [`ClipboardContent`] intentionally does **not**:
//! - Describe MIME types or payload formats
//! - Contain representation-specific attributes such as size or encoding
//! - Encode transport, storage, or UI-specific behavior
//!
//! ## ClipboardItem (representation level)
//!
//! [`ClipboardItem`] represents a single, concrete clipboard representation
//! identified by its MIME type.
//!
//! Each item describes **how** the same clipboard snapshot can be interpreted
//! or consumed by downstream systems.
//!
//! Responsibilities of [`ClipboardItem`] include:
//! - Declaring the MIME type of the representation
//! - Holding the normalized payload data
//! - Exposing representation-specific, derived metadata (e.g. payload size)
//!
//! [`ClipboardItem`] intentionally does **not**:
//! - Carry timestamps or event-level context
//! - Act as a standalone clipboard event
//! - Make assumptions about transport, sync, or persistence semantics
//!
//! ## Metadata (`meta`) and extensibility
//!
//! Both [`ClipboardContent`] and [`ClipboardItem`] expose a `meta` field for
//! optional, non-semantic attributes.
//!
//! Metadata is treated as **hints**, not as part of the core clipboard model:
//! - Metadata keys are not required to be present
//! - Unknown metadata must be safely ignored
//! - Metadata does not participate in content identity or deduplication
//!
//! This design allows platform adapters and infrastructure layers to attach
//! additional information (such as observed payload size or source hints)
//! without affecting the stability of the core model.
//!
//! ## Guiding principle
//!
//! In short:
//!
//! - [`ClipboardContent`] answers: **"What was copied, and when?"**
//! - [`ClipboardItem`] answers: **"In which concrete forms can it be represented?"**
//!
//! Keeping these responsibilities separate is essential for long-term
//! extensibility, cross-platform support, and stable synchronization semantics.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::clipboard::MimeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Blake3V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayloadHash {
    pub alg: HashAlgorithm,
    pub bytes: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemHash {
    pub alg: HashAlgorithm,
    pub bytes: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    pub alg: HashAlgorithm,
    pub bytes: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClipboardData {
    /// UTF-8 text
    Text { text: String },

    /// raw bytes (image, rtf, files, etc.)
    Bytes { bytes: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardItem {
    /// MIME type, e.g. "text/plain", "image/png"
    pub mime: MimeType,

    /// payload
    pub data: ClipboardData,

    /// optional hints
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimestampMs(i64);

impl TimestampMs {
    /// Unix epoch milliseconds (UTC)
    pub fn from_epoch_millis(ms: i64) -> Self {
        Self(ms)
    }

    pub fn as_millis(&self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardContent {
    /// schema version, fixed = 1
    pub v: u32,

    /// unix epoch millis
    pub occurred_at: TimestampMs,

    /// one clipboard snapshot may contain multiple representations
    pub items: Vec<ClipboardItem>,
    
    pub device_id: String,
    
    pub origin: ClipboardOrigin,
}

impl ClipboardItem {
    /// Returns the size of the payload **as currently held in memory**.
    ///
    /// This value reflects the size of the Rust representation only:
    /// - `Text` returns the UTF-8 byte length of the `String`
    /// - `Bytes` returns the length of the in-memory `Vec<u8>`
    ///
    /// ## Important
    /// - This is **not** the actual size of the clipboard data as provided
    ///   by the operating system.
    /// - This value may be smaller than the real clipboard payload,
    ///   or even zero for placeholder / lazy-loaded items.
    ///
    /// ## Intended use
    /// - Debugging and logging
    /// - Tests and in-memory inspections
    /// - Estimating heap usage of the current process
    ///
    /// ## Not suitable for
    /// - Sync or transport decisions
    /// - Bandwidth estimation
    /// - Storage quota or eviction policies
    ///
    /// See [`size_bytes`] for the actual clipboard payload size.
    pub fn data_len(&self) -> usize {
        match &self.data {
            ClipboardData::Text { text } => text.len(),
            ClipboardData::Bytes { bytes } => bytes.len(),
        }
    }

    /// Returns the **actual size in bytes of the clipboard payload**
    /// as observed at the platform or transport boundary.
    ///
    /// This value represents the real size of the data as it existed
    /// in the system clipboard or as transmitted over the network,
    /// and is typically populated by the platform adapter.
    ///
    /// The size is stored as a derived attribute under the
    /// `sys.size_bytes` metadata key and is intentionally decoupled
    /// from the in-memory representation.
    ///
    /// ## Why this is not derived from `data_len`
    /// - Clipboard data may be normalized, transcoded, or partially loaded
    /// - Some items may use lazy loading or external blob references
    /// - Multiple clipboard representations (e.g. text + rtf + html)
    ///   may exist with different sizes
    ///
    /// ## Intended use
    /// - Sync and transport size gating
    /// - Bandwidth and latency estimation
    /// - Storage limits and eviction strategies
    /// - User-facing size display
    ///
    /// ## Returns
    /// - `Some(bytes)` if the platform provided a size hint
    /// - `None` if the size is unknown or intentionally omitted
    pub fn size_bytes(&self) -> Option<u64> {
        self.meta.get("sys.size_bytes")?.parse().ok()
    }

    pub fn hash(&self) -> ItemHash {
        ItemHash::compute(self)
    }
}

impl ClipboardContent {
    /// Computes a hexadecimal deduplication hash for this clipboard snapshot.
    ///
    /// The hash is derived deterministically from the snapshot's schema version and its items and
    /// intended for use as a deduplication key.
    ///
    /// # Returns
    ///
    /// A lowercase hexadecimal string representing the computed hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let content = ClipboardContent {
    ///     v: 1,
    ///     ts_ms: 0,
    ///     items: Vec::new(),
    ///     meta: BTreeMap::new(),
    /// };
    /// let h1 = content.content_hash();
    /// let h2 = content.content_hash();
    /// assert_eq!(h1, h2);
    /// ```
    pub fn content_hash(&self) -> ContentHash {
        ContentHash::compute(&self.items)
    }
}

impl PayloadHash {
    pub fn compute(data: &ClipboardData) -> Self {
        match data {
            ClipboardData::Text { text } => Self::hash_bytes(text.as_bytes()),
            ClipboardData::Bytes { bytes } => Self::hash_bytes(bytes),
        }
    }

    fn hash_bytes(input: &[u8]) -> Self {
        let hash = blake3::hash(input);
        Self {
            alg: HashAlgorithm::Blake3V1,
            bytes: hash.into(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl ItemHash {
    pub fn compute(item: &ClipboardItem) -> Self {
        let payload_hash = PayloadHash::compute(&item.data);

        let mut buf = Vec::new();

        // 1. mime（稳定字符串）
        buf.extend_from_slice(item.mime.as_str().as_bytes());
        buf.push(0);

        // 2. data kind（语义必须参与）
        match &item.data {
            ClipboardData::Text { .. } => buf.extend_from_slice(b"text"),
            ClipboardData::Bytes { .. } => buf.extend_from_slice(b"bytes"),
        }
        buf.push(0);

        // 3. payload hash（固定 32 bytes）
        buf.extend_from_slice(&payload_hash.bytes);

        let hash = blake3::hash(&buf);

        Self {
            alg: HashAlgorithm::Blake3V1,
            bytes: hash.into(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.as_bytes())
    }
}

impl ContentHash {
    pub fn compute(items: &[ClipboardItem]) -> Self {
        let mut item_hashes: Vec<[u8; 32]> = items
            .iter()
            .map(|item| ItemHash::compute(item).bytes)
            .collect();

        // 顺序无关（非常重要）
        item_hashes.sort();

        let mut buf = Vec::new();
        for h in item_hashes {
            buf.extend_from_slice(&h);
        }

        let hash = blake3::hash(&buf);

        Self {
            alg: HashAlgorithm::Blake3V1,
            bytes: hash.into(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.as_bytes())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipboardOrigin {
    Local,
    Remote,
}

impl ClipboardOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipboardOrigin::Local => "local",
            ClipboardOrigin::Remote => "remote",
        }
    }
}

impl From<&str> for ClipboardOrigin {
    /// Create a ClipboardOrigin from a string slice.
    ///
    /// Maps the string `"local"` to `ClipboardOrigin::Local` and `"remote"` to
    /// `ClipboardOrigin::Remote`. Any other value yields `ClipboardOrigin::Local`.
    ///
    /// # Examples
    ///
    /// ```
    /// let origin = ClipboardOrigin::from("remote");
    /// assert_eq!(origin, ClipboardOrigin::Remote);
    /// ```
    fn from(s: &str) -> Self {
        match s {
            "local" => ClipboardOrigin::Local,
            "remote" => ClipboardOrigin::Remote,
            _ => ClipboardOrigin::Local, // Default to Local for unknown values
        }
    }
}

impl From<String> for ClipboardOrigin {
    /// Converts an owned string into a `ClipboardOrigin`.
    ///
    /// Maps the exact string `"local"` to `ClipboardOrigin::Local` and `"remote"` to
    /// `ClipboardOrigin::Remote`. Any other value defaults to `ClipboardOrigin::Local`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::clipboard::view::ClipboardOrigin;
    ///
    /// let o1 = ClipboardOrigin::from("local".to_string());
    /// assert_eq!(o1, ClipboardOrigin::Local);
    ///
    /// let o2 = ClipboardOrigin::from("remote".to_string());
    /// assert_eq!(o2, ClipboardOrigin::Remote);
    ///
    /// let o3 = ClipboardOrigin::from("unknown".to_string());
    /// assert_eq!(o3, ClipboardOrigin::Local);
    /// ```
    fn from(s: String) -> Self {
        match s.as_str() {
            "local" => ClipboardOrigin::Local,
            "remote" => ClipboardOrigin::Remote,
            _ => ClipboardOrigin::Local, // Default to Local for unknown values
        }
    }
}
