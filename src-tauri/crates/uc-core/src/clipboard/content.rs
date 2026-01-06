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
use std::hash::{Hash, Hasher};

use crate::clipboard::meta_keys;
use crate::clipboard::MimeType;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardContent {
    /// schema version, fixed = 1
    pub v: u32,

    /// unix epoch millis
    pub ts_ms: i64,

    /// one clipboard snapshot may contain multiple representations
    pub items: Vec<ClipboardItem>,

    /// reserved for forward compatibility
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
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
    pub fn content_hash(&self) -> String {
        use std::hash::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.v.hash(&mut hasher);
        self.items.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Retrieve the device identifier stored in the snapshot's metadata.
    ///
    /// Looks up the metadata entry keyed by `meta_keys::sys::DEVICE_ID` and returns its string value if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use crate::clipboard::meta_keys;
    /// // Construct a minimal ClipboardContent with the device id in meta.
    /// let mut meta = BTreeMap::new();
    /// meta.insert(meta_keys::sys::DEVICE_ID.to_string(), "device-123".to_string());
    /// let content = ClipboardContent { v: 1, ts_ms: 0, items: vec![], meta };
    /// assert_eq!(content.get_device_id(), Some("device-123"));
    /// ```
    pub fn get_device_id(&self) -> Option<&str> {
        self.meta.get(meta_keys::sys::DEVICE_ID).map(|s| s.as_str())
    }

    /// Returns the origin identifier stored in the content's metadata, if present.
    ///
    /// The origin is read from the metadata key `meta_keys::sys::ORIGIN`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use crate::clipboard::{meta_keys, ClipboardContent, ClipboardItem, ClipboardData, MimeType};
    ///
    /// let mut meta = BTreeMap::new();
    /// meta.insert(meta_keys::sys::ORIGIN.to_string(), "remote-device".to_string());
    ///
    /// let content = ClipboardContent {
    ///     v: 1,
    ///     ts_ms: 0,
    ///     items: Vec::new(),
    ///     meta,
    /// };
    ///
    /// assert_eq!(content.get_origin(), Some("remote-device"));
    /// ```
    pub fn get_origin(&self) -> Option<&str> {
        self.meta.get(meta_keys::sys::ORIGIN).map(|s| s.as_str())
    }
}

impl Hash for ClipboardData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ClipboardData::Text { text } => {
                0u8.hash(state);
                text.hash(state);
            }
            ClipboardData::Bytes { bytes } => {
                1u8.hash(state);
                bytes.hash(state);
            }
        }
    }
}

impl Hash for ClipboardItem {
    /// Feeds the clipboard item's identity into the provided hasher by hashing its MIME type and payload.
    ///
    /// This implementation produces a deterministic hash based on the `mime` and `data` fields so that
    /// two `ClipboardItem` instances with equal MIME and payload produce the same hash.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::hash_map::DefaultHasher;
    /// use std::hash::{Hash, Hasher};
    ///
    /// // Construct a sample ClipboardItem (types assumed to be in scope)
    /// let item = ClipboardItem {
    ///     mime: "text/plain".parse().unwrap(),
    ///     data: ClipboardData::Text("hello".into()),
    ///     meta: Default::default(),
    /// };
    ///
    /// let mut hasher = DefaultHasher::new();
    /// item.hash(&mut hasher);
    /// let _hash = hasher.finish();
    /// ```
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mime.hash(state);
        self.data.hash(state);
    }
}
