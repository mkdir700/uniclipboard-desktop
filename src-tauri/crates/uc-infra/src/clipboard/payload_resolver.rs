//! Clipboard Payload Resolver Implementation
//!
//! Resolves persisted clipboard representations into usable payloads.
//! Read-only: returns inline data, blob references, or cache/spool bytes.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info_span, warn, Instrument};

use uc_core::clipboard::{PayloadAvailability, PersistedClipboardRepresentation};
use uc_core::ids::RepresentationId;
use uc_core::ports::clipboard::ResolvedClipboardPayload;
use uc_core::ports::ClipboardPayloadResolverPort;

use crate::clipboard::{RepresentationCache, SpoolManager};

/// Clipboard payload resolver implementation
pub struct ClipboardPayloadResolver {
    cache: Arc<RepresentationCache>,
    spool: Arc<SpoolManager>,
    worker_tx: mpsc::Sender<RepresentationId>,
}

impl ClipboardPayloadResolver {
    pub fn new(
        cache: Arc<RepresentationCache>,
        spool: Arc<SpoolManager>,
        worker_tx: mpsc::Sender<RepresentationId>,
    ) -> Self {
        Self {
            cache,
            spool,
            worker_tx,
        }
    }
}

#[async_trait]
impl ClipboardPayloadResolverPort for ClipboardPayloadResolver {
    async fn resolve(
        &self,
        representation: &PersistedClipboardRepresentation,
    ) -> Result<ResolvedClipboardPayload> {
        let span = info_span!(
            "infra.payload.resolve",
            representation_id = %representation.id,
            format_id = %representation.format_id,
        );
        async move {
            let mime = Self::mime_or_default(representation);

            match &representation.payload_state {
                PayloadAvailability::Inline => {
                    let inline_data = match representation.inline_data.as_ref() {
                        Some(bytes) => bytes,
                        None => {
                            let err = anyhow::anyhow!(
                                "payload_state Inline but inline_data is None for {}",
                                representation.id
                            );
                            error!(
                                representation_id = %representation.id,
                                error = %err,
                                "Inline payload is missing inline_data"
                            );
                            return Err(err);
                        }
                    };
                    debug!("Resolving from inline data");
                    Ok(ResolvedClipboardPayload::Inline {
                        mime,
                        bytes: inline_data.clone(),
                    })
                }
                PayloadAvailability::BlobReady => {
                    let blob_id = match representation.blob_id.as_ref() {
                        Some(id) => id,
                        None => {
                            let err = anyhow::anyhow!(
                                "payload_state BlobReady but blob_id is None for {}",
                                representation.id
                            );
                            error!(
                                representation_id = %representation.id,
                                error = %err,
                                "BlobReady payload is missing blob_id"
                            );
                            return Err(err);
                        }
                    };
                    debug!("Resolving from existing blob reference");
                    Ok(ResolvedClipboardPayload::BlobRef {
                        mime,
                        blob_id: blob_id.clone(),
                    })
                }
                PayloadAvailability::Staged
                | PayloadAvailability::Processing
                | PayloadAvailability::Failed { .. } => {
                    if let Some(bytes) = self.cache.get(&representation.id).await {
                        debug!("Resolving from cache bytes");
                        self.try_requeue(&representation.id);
                        return Ok(ResolvedClipboardPayload::Inline { mime, bytes });
                    }

                    match self.spool.read(&representation.id).await {
                        Ok(Some(bytes)) => {
                            debug!("Resolving from spool bytes");
                            self.try_requeue(&representation.id);
                            Ok(ResolvedClipboardPayload::Inline { mime, bytes })
                        }
                        Ok(None) => {
                            warn!(
                                representation_id = %representation.id,
                                payload_state = ?&representation.payload_state,
                                "Bytes not available in cache or spool"
                            );
                            Err(anyhow::anyhow!(
                                "payload bytes not available for {}",
                                representation.id
                            ))
                        }
                        Err(err) => {
                            error!(
                                representation_id = %representation.id,
                                error = %err,
                                "Failed to read bytes from spool"
                            );
                            Err(err)
                        }
                    }
                }
                PayloadAvailability::Lost => {
                    let details = representation
                        .last_error
                        .as_deref()
                        .unwrap_or("payload marked as lost");
                    Err(anyhow::anyhow!(
                        "payload is lost for {}: {}",
                        representation.id,
                        details
                    ))
                }
            }
        }
        .instrument(span)
        .await
    }
}

impl ClipboardPayloadResolver {
    fn mime_or_default(representation: &PersistedClipboardRepresentation) -> String {
        representation
            .mime_type
            .clone()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string())
    }

    fn try_requeue(&self, rep_id: &RepresentationId) {
        if let Err(err) = self.worker_tx.try_send(rep_id.clone()) {
            warn!(
                representation_id = %rep_id,
                error = %err,
                "Failed to re-queue representation for background processing"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::time::Duration;

    use tempfile::tempdir;
    use uc_core::ids::{BlobId, FormatId, RepresentationId};
    use uc_core::MimeType;

    #[tokio::test]
    async fn test_resolve_inline_returns_inline_payload() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let temp_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1024)?);
        let (worker_tx, _worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache, spool, worker_tx);

        let bytes = b"hello".to_vec();
        let rep = PersistedClipboardRepresentation::new(
            RepresentationId::new(),
            FormatId::from("public.utf8-plain-text"),
            Some(MimeType::text_plain()),
            bytes.len() as i64,
            Some(bytes.clone()),
            None,
        );

        let payload = resolver.resolve(&rep).await?;
        match payload {
            ResolvedClipboardPayload::Inline { mime, bytes: out } => {
                assert_eq!(mime, "text/plain");
                assert_eq!(out, bytes);
            }
            _ => panic!("Expected inline payload"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_blob_ready_returns_blob_ref() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let temp_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1024)?);
        let (worker_tx, _worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache, spool, worker_tx);

        let blob_id = BlobId::new();
        let rep = PersistedClipboardRepresentation::new(
            RepresentationId::new(),
            FormatId::from("public.png"),
            Some(MimeType::from_str("image/png")?),
            10,
            None,
            Some(blob_id.clone()),
        );

        let payload = resolver.resolve(&rep).await?;
        match payload {
            ResolvedClipboardPayload::BlobRef { mime, blob_id: out } => {
                assert_eq!(mime, "image/png");
                assert_eq!(out, blob_id);
            }
            _ => panic!("Expected blob reference payload"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_staged_uses_cache_and_requeues() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let temp_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1024)?);
        let (worker_tx, mut worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache.clone(), spool, worker_tx);

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];
        cache.put(&rep_id, bytes.clone()).await;

        let rep = PersistedClipboardRepresentation::new_staged(
            rep_id.clone(),
            FormatId::from("public.png"),
            Some(MimeType::from_str("image/png")?),
            bytes.len() as i64,
        );

        let payload = resolver.resolve(&rep).await?;
        match payload {
            ResolvedClipboardPayload::Inline { bytes: out, .. } => {
                assert_eq!(out, bytes);
            }
            _ => panic!("Expected inline payload from cache"),
        }

        let requeued = tokio::time::timeout(Duration::from_millis(50), worker_rx.recv()).await?;
        assert_eq!(requeued, Some(rep_id));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_staged_uses_spool_and_requeues() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let spool_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(spool_dir.path(), 1024)?);
        let (worker_tx, mut worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache, spool.clone(), worker_tx);

        let rep_id = RepresentationId::new();
        let bytes = vec![9, 8, 7];
        spool.write(&rep_id, &bytes).await?;

        let rep = PersistedClipboardRepresentation::new_staged(
            rep_id.clone(),
            FormatId::from("public.png"),
            Some(MimeType::from_str("image/png")?),
            bytes.len() as i64,
        );

        let payload = resolver.resolve(&rep).await?;
        match payload {
            ResolvedClipboardPayload::Inline { bytes: out, .. } => {
                assert_eq!(out, bytes);
            }
            _ => panic!("Expected inline payload from spool"),
        }

        let requeued = tokio::time::timeout(Duration::from_millis(50), worker_rx.recv()).await?;
        assert_eq!(requeued, Some(rep_id));
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_staged_missing_bytes_returns_error() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let temp_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1024)?);
        let (worker_tx, _worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache, spool, worker_tx);

        let rep = PersistedClipboardRepresentation::new_staged(
            RepresentationId::new(),
            FormatId::from("public.png"),
            Some(MimeType::from_str("image/png")?),
            10,
        );

        let result = resolver.resolve(&rep).await;
        assert!(result.is_err(), "Expected error when bytes missing");
        Ok(())
    }

    #[tokio::test]
    async fn test_resolve_lost_returns_error() -> Result<()> {
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let temp_dir = tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1024)?);
        let (worker_tx, _worker_rx) = mpsc::channel(1);
        let resolver = ClipboardPayloadResolver::new(cache, spool, worker_tx);

        let rep = PersistedClipboardRepresentation::new_with_state(
            RepresentationId::new(),
            FormatId::from("public.png"),
            Some(MimeType::from_str("image/png")?),
            10,
            None,
            None,
            PayloadAvailability::Lost,
            Some("missing".to_string()),
        )?;

        let result = resolver.resolve(&rep).await;
        assert!(result.is_err(), "Expected error when payload is lost");
        Ok(())
    }
}
