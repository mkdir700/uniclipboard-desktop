use super::super::common::CommonClipboardImpl;
use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::ClipboardContext;
use std::sync::{Arc, Mutex};
use tracing::{debug, debug_span, error};
use uc_core::clipboard::SystemClipboardSnapshot;
use uc_core::ports::SystemClipboardPort;

pub struct LinuxClipboard {
    inner: Arc<Mutex<ClipboardContext>>,
}

impl LinuxClipboard {
    pub fn new() -> Result<Self> {
        let context = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(context)),
        })
    }
}

#[async_trait]
impl SystemClipboardPort for LinuxClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        let span = debug_span!("platform.linux.read_clipboard");
        span.in_scope(|| {
            let mut ctx = match self.inner.lock() {
                Ok(ctx) => ctx,
                Err(poison) => {
                    error!("Failed to lock clipboard context (poisoned mutex)");
                    return Err(anyhow::anyhow!(
                        "Clipboard mutex poisoned: {}",
                        poison.to_string()
                    ));
                }
            };
            let snapshot = CommonClipboardImpl::read_snapshot(&mut ctx)?;

            debug!(
                formats = snapshot.representations.len(),
                total_size_bytes = snapshot.total_size_bytes(),
                "Captured system clipboard snapshot"
            );

            Ok(snapshot)
        })
    }

    fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
        let span = debug_span!(
            "platform.linux.write_clipboard",
            representations = snapshot.representations.len(),
        );
        span.in_scope(|| {
            let mut ctx = self.inner.lock().map_err(|poison| {
                error!("Failed to lock clipboard context in write_snapshot (poisoned mutex)");
                anyhow::anyhow!(
                    "mutex poisoned locking inner in write_snapshot: {}",
                    poison.to_string()
                )
            })?;
            CommonClipboardImpl::write_snapshot(&mut ctx, snapshot)?;

            debug!("Wrote clipboard snapshot to system");
            Ok(())
        })
    }
}
