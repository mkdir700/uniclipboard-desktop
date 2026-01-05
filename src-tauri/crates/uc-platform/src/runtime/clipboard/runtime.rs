//! clipboard_runtime.rs
//!
//! A minimal, cross-platform clipboard runtime implementation.
//!
//! This runtime is responsible for lifecycle management (start/stop) and
//! bridging OS clipboard changes into the uc-platform event bus by driving
//! a ClipboardWatcher in a background task.
//!
//! Current implementation uses polling (tokio interval). Platform-specific
//! event-driven runtimes can replace this without changing upper layers.

use anyhow::Result;
use async_trait::async_trait;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::interval,
};

use crate::ports::ClipboardRuntimePort;
use crate::{ipc::PlatformEvent, runtime::clipboard::ClipboardWatcher};
use uc_core::ports::ClipboardPort;

pub struct PollingClipboardRuntime<C>
where
    C: ClipboardPort,
{
    watcher: Arc<ClipboardWatcher<C>>,
    running: AtomicBool,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl<C> PollingClipboardRuntime<C>
where
    C: ClipboardPort,
{
    pub fn new(watcher: Arc<ClipboardWatcher<C>>) -> Self {
        Self {
            watcher,
            running: AtomicBool::new(false),
            handle: Mutex::new(None),
        }
    }
}

#[async_trait]
impl<C> ClipboardRuntimePort for PollingClipboardRuntime<C>
where
    C: ClipboardPort + 'static,
{
    async fn start(&self, _tx: mpsc::Sender<PlatformEvent>) -> Result<()> {
        if self
            .running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(()); // 幂等
        }

        let watcher = self.watcher.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(500));

            loop {
                ticker.tick().await;

                if let Err(err) = watcher.check_once().await {
                    log::warn!("clipboard check failed: {:?}", err);
                }
            }
        });

        *self.handle.lock().await = Some(handle);

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        if !self.running.swap(false, Ordering::AcqRel) {
            return Ok(());
        }

        if let Some(handle) = self.handle.lock().await.take() {
            handle.abort(); // polling 是可直接 abort 的
        }

        Ok(())
    }
}
