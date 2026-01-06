//! Clipboard watcher runtime module
//!
//! This module provides a long-running clipboard watcher used by the
//! `uc-platform` runtime to observe local clipboard changes and emit
//! platform-level events.
//!
//! ## Responsibility
//!
//! The clipboard watcher is responsible for:
//!
//! - Periodically reading the local clipboard via `ClipboardPort`
//! - Detecting meaningful clipboard changes (using content hashing)
//! - Emitting `PlatformEvent::ClipboardChanged` events into the runtime event bus
//!
//! The watcher **does not**:
//!
//! - Perform business decisions (e.g. whether to sync or store data)
//! - Interact with network or storage layers
//! - Modify clipboard content beyond observing it
//!
//! ## Architecture Position
//!
//! The watcher is a runtime-level component and acts as an **event source**.
//! It bridges platform-specific clipboard behavior with platform-agnostic
//! use cases by translating clipboard state changes into structured
//! `PlatformEvent`s.
//!
//! ```text
//! Local Clipboard
//!      ↓
//! ClipboardPort
//!      ↓
//! ClipboardWatcher   (this module)
//!      ↓
//! PlatformEvent
//!      ↓
//! Runtime Event Loop
//!      ↓
//! Use Cases
//! ```
//!
//! ## Change Detection
//!
//! Clipboard changes are detected by computing a stable hash of the
//! clipboard content and comparing it with the previously observed value.
//! Identical consecutive contents are ignored to avoid redundant events.
//!
//! This design:
//!
//! - Prevents event storms during high-frequency clipboard updates
//! - Provides a simple foundation for loop prevention and debouncing
//!
//! ## Lifecycle
//!
//! A `ClipboardWatcher` is constructed via `new()` and started by calling
//! `run()`, which enters an asynchronous polling loop.
//!
//! The watcher is intended to be spawned and managed by the runtime,
//! not instantiated or controlled directly by use cases.
//!
//! ## Design Notes
//!
//! - Polling is used as the default observation strategy for portability
//!   and reliability across platforms.
//! - Platform-native event-based implementations may be introduced in
//!   the future without changing the public interface.
//! - The watcher emits complete event payloads rather than signals,
//!   ensuring downstream components remain stateless and deterministic.
//!
//! This module is intentionally minimal and focused, serving as a stable
//! foundation for clipboard-driven workflows within the platform runtime.
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use uc_core::ports::LocalClipboardPort;

use crate::ipc::PlatformEvent;

pub struct ClipboardWatcher<C>
where
    C: LocalClipboardPort,
{
    clipboard: Arc<C>,
    tx: mpsc::Sender<PlatformEvent>,
    last_hash: Mutex<Option<String>>,
    ignore_next_hash: Mutex<Option<String>>,
}

impl<C> ClipboardWatcher<C>
where
    C: LocalClipboardPort + 'static,
{
    pub fn new(clipboard: Arc<C>, tx: mpsc::Sender<PlatformEvent>) -> Self {
        Self {
            clipboard,
            tx,
            last_hash: Mutex::new(None),
            ignore_next_hash: Mutex::new(None),
        }
    }

    /// Check clipboard once.
    ///
    /// This method is idempotent and side-effect free if content
    /// has not meaningfully changed.
    pub async fn check_once(&self) -> Result<()> {
        let content = self.clipboard.read().await?;

        let hash = content.content_hash(); // 假设你已有稳定 hash

        // 1️⃣ 忽略“远端写入”的回环
        if self.should_ignore(&hash).await {
            return Ok(());
        }

        // 2️⃣ 判断是否真的变化
        let mut last = self.last_hash.lock().await;
        if last.as_deref() == Some(&hash) {
            return Ok(());
        }

        *last = Some(hash.clone());

        // 3️⃣ Emit event
        self.tx
            .send(PlatformEvent::ClipboardChanged { content })
            .await?;

        Ok(())
    }

    async fn should_ignore(&self, hash: &str) -> bool {
        let mut guard = self.ignore_next_hash.lock().await;
        if guard.as_deref() == Some(hash) {
            *guard = None;
            true
        } else {
            false
        }
    }

    /// Mark a clipboard write as originating from a remote source.
    ///
    /// # Purpose
    ///
    /// This method is used to **prevent clipboard sync loops**.
    ///
    /// When clipboard content is written as a result of a **remote / network
    /// synchronization**, the system clipboard will still change locally.
    /// Without additional context, the `ClipboardWatcher` would detect this
    /// change and incorrectly treat it as a **local user action**, emitting
    /// a new `PlatformEvent::ClipboardChanged` and causing a sync loop.
    ///
    /// Calling `mark_remote_write` informs the watcher that **the next clipboard
    /// change with the given content hash should be ignored once**.
    ///
    /// # Correct Usage (IMPORTANT)
    ///
    /// **This method MUST be called *before* writing to the clipboard.**
    ///
    /// ```rust,no_run
    /// let hash = content.content_hash();
    ///
    /// // 1. Mark the upcoming clipboard change as remote
    /// clipboard_watcher.mark_remote_write(hash.clone()).await;
    ///
    /// // 2. Write content to the system clipboard
    /// clipboard.write(content).await?;
    /// ```
    ///
    /// Reversing this order will result in the watcher emitting a
    /// `ClipboardChanged` event for a remote write, causing infinite
    /// synchronization loops.
    ///
    /// # Semantics
    ///
    /// - The ignore marker applies to **one clipboard change only**
    /// - Matching is done by **content hash**
    /// - After a match is observed, the marker is automatically cleared
    ///
    /// # Scope
    ///
    /// This method is intentionally **not part of `ClipboardPort`**.
    /// It is an internal coordination mechanism between the runtime and
    /// the clipboard watcher, preserving a clean separation between
    /// platform I/O and synchronization logic.
    pub async fn mark_remote_write(&self, hash: String) {
        let mut guard = self.ignore_next_hash.lock().await;
        *guard = Some(hash);
    }
}
