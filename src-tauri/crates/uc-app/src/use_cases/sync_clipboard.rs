//! SyncClipboard use case - handles clipboard synchronization

use anyhow::Result;
use log::{debug, info};
use std::sync::Arc;
use uc_core::clipboard::Payload;
use uc_core::decision::DomainDecision;
use uc_core::network::ClipboardMessage;
use uc_core::ports::{LocalClipboardPort, NetworkPort, StoragePort};
use uc_core::sync::{SyncDomain, SyncEvent};

/// SyncClipboard use case - handles clipboard synchronization
pub struct SyncClipboard<N, C, S>
where
    N: NetworkPort,
    C: LocalClipboardPort,
    S: StoragePort,
{
    domain: SyncDomain,
    network: Arc<N>,
    clipboard: Arc<C>,
    storage: Arc<S>,
}

impl<N, C, S> SyncClipboard<N, C, S>
where
    N: NetworkPort,
    C: LocalClipboardPort,
    S: StoragePort,
{
    pub fn new(domain: SyncDomain, network: Arc<N>, clipboard: Arc<C>, storage: Arc<S>) -> Self {
        Self {
            domain,
            network,
            clipboard,
            storage,
        }
    }

    /// Handle local clipboard change - broadcast to peers
    pub async fn on_local_change(&mut self, payload: Payload) -> Result<()> {
        info!("Local clipboard changed: {:?}", payload.content_type());

        let event = SyncEvent::LocalClipboardChanged { payload };
        let decision = self.domain.apply(event);
        self.execute_decision(decision).await
    }

    /// Handle remote clipboard message - apply locally
    pub async fn on_remote_message(&self, msg: ClipboardMessage) -> Result<()> {
        info!("Remote clipboard received from: {}", msg.origin_device_name);

        // 1. Check for duplicates
        if self.is_duplicate(&msg).await? {
            debug!("Duplicate clipboard message, ignoring");
            return Ok(());
        }

        // TODO:
        // let event = SyncEvent::RemoteClipboardReceived { payload: (), origin: (), content_hash: () };
        // let decision = self.domain.apply(event);
        // self.execute_decision(decision).await

        Ok(())
    }

    async fn is_duplicate(&self, _msg: &ClipboardMessage) -> Result<bool> {
        // TODO: Implement duplicate check based on content_hash
        Ok(false)
    }

    async fn execute_decision(&self, decision: DomainDecision) -> Result<()> {
        match decision {
            DomainDecision::Ignore => {
                debug!("Domain decided to ignore");
            }

            DomainDecision::PersistLocalClipboard { content: payload } => {
                // TODO: self.storage.save(payload).await?
            }

            DomainDecision::BroadcastClipboard { content: payload } => {
                // TODO: self.network.broadcast_clipboard(payload).await?
            }

            DomainDecision::ApplyRemoteClipboard {
                content: payload, ..
            } => {
                // TODO: self.clipboard.write(payload).await?
            }

            DomainDecision::EnterConflict { .. } => {
                // TODO: notify UI / log
            }

            other => {
                debug!("Unhandled decision: {:?}", other);
            }
        }

        Ok(())
    }
}
