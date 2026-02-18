use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, info, info_span, warn, Instrument};
use uc_core::ids::{FormatId, RepresentationId};
use uc_core::network::protocol::ClipboardTextPayloadV1;
use uc_core::network::ClipboardMessage;
use uc_core::ports::{
    ClipboardChangeOriginPort, DeviceIdentityPort, EncryptionPort, EncryptionSessionPort,
    SystemClipboardPort,
};
use uc_core::security::{aad, model::EncryptedBlob};
use uc_core::{
    ClipboardChangeOrigin, MimeType, ObservedClipboardRepresentation, SystemClipboardSnapshot,
};

pub struct SyncInboundClipboardUseCase {
    local_clipboard: Arc<dyn SystemClipboardPort>,
    clipboard_change_origin: Arc<dyn ClipboardChangeOriginPort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
    encryption: Arc<dyn EncryptionPort>,
    device_identity: Arc<dyn DeviceIdentityPort>,
}

impl SyncInboundClipboardUseCase {
    pub fn new(
        local_clipboard: Arc<dyn SystemClipboardPort>,
        clipboard_change_origin: Arc<dyn ClipboardChangeOriginPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
        encryption: Arc<dyn EncryptionPort>,
        device_identity: Arc<dyn DeviceIdentityPort>,
    ) -> Self {
        Self {
            local_clipboard,
            clipboard_change_origin,
            encryption_session,
            encryption,
            device_identity,
        }
    }

    pub async fn execute(&self, message: ClipboardMessage) -> Result<()> {
        let span = info_span!(
            "usecase.clipboard.sync_inbound.execute",
            message_id = %message.id,
            origin_device_id = %message.origin_device_id,
        );

        async move {
            let local_device_id = self.device_identity.current_device_id().to_string();
            if message.origin_device_id == local_device_id {
                debug!("Ignoring inbound clipboard message from local device");
                return Ok(());
            }

            if !self.encryption_session.is_ready().await {
                info!("Skipping inbound apply because encryption session is not ready");
                return Ok(());
            }

            let encrypted_blob: EncryptedBlob = serde_json::from_slice(&message.encrypted_content)
                .context("failed to deserialize encrypted inbound clipboard payload")?;

            let master_key = self
                .encryption_session
                .get_master_key()
                .await
                .map_err(anyhow::Error::from)
                .context("failed to access encryption session master key for inbound apply")?;

            let plaintext = self
                .encryption
                .decrypt_blob(
                    &master_key,
                    &encrypted_blob,
                    &aad::for_network_clipboard(&message.id),
                )
                .await
                .map_err(anyhow::Error::from)
                .context("failed to decrypt inbound clipboard payload")?;

            let payload: ClipboardTextPayloadV1 = serde_json::from_slice(&plaintext)
                .context("failed to deserialize inbound clipboard payload")?;
            if !is_text_plain_mime(&payload.mime) {
                warn!(mime = %payload.mime, "Skipping inbound apply because payload mime is not text/plain");
                return Ok(());
            }

            let current_snapshot = self
                .local_clipboard
                .read_snapshot()
                .context("failed to read local clipboard snapshot for inbound dedupe")?;
            let already_applied = current_snapshot
                .representations
                .iter()
                .any(|rep| rep.content_hash().to_string() == message.content_hash);
            if already_applied {
                debug!("Skipping inbound apply because local clipboard already matches content hash");
                return Ok(());
            }

            let snapshot = SystemClipboardSnapshot {
                ts_ms: payload.ts_ms,
                representations: vec![ObservedClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: FormatId::from("text"),
                    mime: Some(MimeType::text_plain()),
                    bytes: payload.text.into_bytes(),
                }],
            };

            self.clipboard_change_origin
                .set_next_origin(
                    ClipboardChangeOrigin::RemotePush,
                    Duration::from_millis(REMOTE_PUSH_ORIGIN_TTL_MS),
                )
                .await;

            if let Err(err) = self.local_clipboard.write_snapshot(snapshot) {
                self.clipboard_change_origin
                    .consume_origin_or_default(ClipboardChangeOrigin::LocalCapture)
                    .await;
                return Err(err).context("failed to write inbound clipboard snapshot");
            }

            info!("Inbound clipboard message applied");
            Ok(())
        }
        .instrument(span)
        .await
    }
}

const REMOTE_PUSH_ORIGIN_TTL_MS: u64 = 100;

fn is_text_plain_mime(mime: &str) -> bool {
    let normalized = mime.trim();
    normalized.eq_ignore_ascii_case(ClipboardTextPayloadV1::MIME_TEXT_PLAIN)
        || normalized.to_ascii_lowercase().starts_with("text/plain;")
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::Utc;
    use uc_core::ids::{FormatId, RepresentationId};
    use uc_core::network::protocol::ClipboardTextPayloadV1;
    use uc_core::security::model::{
        EncryptedBlob, EncryptionAlgo, EncryptionError, EncryptionFormatVersion, KdfParams, Kek,
        MasterKey, Passphrase,
    };
    use uc_core::{
        ClipboardChangeOrigin, DeviceId, MimeType, ObservedClipboardRepresentation,
        SystemClipboardSnapshot,
    };

    struct MockSystemClipboard {
        reads: SystemClipboardSnapshot,
        writes: Arc<Mutex<Vec<SystemClipboardSnapshot>>>,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl SystemClipboardPort for MockSystemClipboard {
        fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
            Ok(self.reads.clone())
        }

        fn write_snapshot(&self, snapshot: SystemClipboardSnapshot) -> Result<()> {
            self.calls
                .lock()
                .expect("calls lock")
                .push("write_snapshot");
            self.writes.lock().expect("writes lock").push(snapshot);
            Ok(())
        }
    }

    struct MockChangeOrigin {
        calls: Arc<Mutex<Vec<&'static str>>>,
        values: Arc<Mutex<Vec<(ClipboardChangeOrigin, Duration)>>>,
    }

    #[async_trait]
    impl ClipboardChangeOriginPort for MockChangeOrigin {
        async fn set_next_origin(&self, origin: ClipboardChangeOrigin, ttl: Duration) {
            self.calls.lock().expect("calls lock").push("set_origin");
            self.values.lock().expect("values lock").push((origin, ttl));
        }

        async fn consume_origin_or_default(
            &self,
            default_origin: ClipboardChangeOrigin,
        ) -> ClipboardChangeOrigin {
            default_origin
        }
    }

    struct MockEncryptionSession {
        ready: bool,
    }

    #[async_trait]
    impl EncryptionSessionPort for MockEncryptionSession {
        async fn is_ready(&self) -> bool {
            self.ready
        }

        async fn get_master_key(&self) -> std::result::Result<MasterKey, EncryptionError> {
            Ok(MasterKey([3; 32]))
        }

        async fn set_master_key(
            &self,
            _master_key: MasterKey,
        ) -> std::result::Result<(), EncryptionError> {
            Ok(())
        }

        async fn clear(&self) -> std::result::Result<(), EncryptionError> {
            Ok(())
        }
    }

    struct MockEncryption {
        decrypt_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl EncryptionPort for MockEncryption {
        async fn derive_kek(
            &self,
            _passphrase: &Passphrase,
            _salt: &[u8],
            _kdf: &KdfParams,
        ) -> std::result::Result<Kek, EncryptionError> {
            Err(EncryptionError::UnsupportedKdfAlgorithm)
        }

        async fn wrap_master_key(
            &self,
            _kek: &Kek,
            _master_key: &MasterKey,
            _aead: EncryptionAlgo,
        ) -> std::result::Result<EncryptedBlob, EncryptionError> {
            Err(EncryptionError::EncryptFailed)
        }

        async fn unwrap_master_key(
            &self,
            _kek: &Kek,
            _wrapped: &EncryptedBlob,
        ) -> std::result::Result<MasterKey, EncryptionError> {
            Err(EncryptionError::WrongPassphrase)
        }

        async fn encrypt_blob(
            &self,
            _master_key: &MasterKey,
            _plaintext: &[u8],
            _aad: &[u8],
            _aead: EncryptionAlgo,
        ) -> std::result::Result<EncryptedBlob, EncryptionError> {
            Err(EncryptionError::EncryptFailed)
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            encrypted: &EncryptedBlob,
            _aad: &[u8],
        ) -> std::result::Result<Vec<u8>, EncryptionError> {
            self.decrypt_calls.fetch_add(1, Ordering::SeqCst);
            Ok(encrypted.ciphertext.clone())
        }
    }

    struct MockDeviceIdentity {
        id: DeviceId,
    }

    impl DeviceIdentityPort for MockDeviceIdentity {
        fn current_device_id(&self) -> DeviceId {
            self.id.clone()
        }
    }

    fn build_text_snapshot(text: &str) -> SystemClipboardSnapshot {
        SystemClipboardSnapshot {
            ts_ms: 1,
            representations: vec![ObservedClipboardRepresentation {
                id: RepresentationId::new(),
                format_id: FormatId::from("text"),
                mime: Some(MimeType::text_plain()),
                bytes: text.as_bytes().to_vec(),
            }],
        }
    }

    fn build_message(text: &str, origin_device_id: &str) -> ClipboardMessage {
        let payload = ClipboardTextPayloadV1::new(text.to_string(), 42);
        let payload_bytes = serde_json::to_vec(&payload).expect("serialize payload");
        let encrypted_blob = EncryptedBlob {
            version: EncryptionFormatVersion::V1,
            aead: EncryptionAlgo::XChaCha20Poly1305,
            nonce: vec![9; 24],
            ciphertext: payload_bytes,
            aad_fingerprint: None,
        };
        let encrypted_content = serde_json::to_vec(&encrypted_blob).expect("serialize blob");
        let content_hash = build_text_snapshot(text).representations[0]
            .content_hash()
            .to_string();

        ClipboardMessage {
            id: "msg-1".to_string(),
            content_hash,
            encrypted_content,
            timestamp: Utc::now(),
            origin_device_id: origin_device_id.to_string(),
            origin_device_name: "peer-device".to_string(),
        }
    }

    fn build_usecase(
        local_snapshot: SystemClipboardSnapshot,
        local_device_id: &str,
        ready: bool,
    ) -> (
        SyncInboundClipboardUseCase,
        Arc<Mutex<Vec<SystemClipboardSnapshot>>>,
        Arc<Mutex<Vec<&'static str>>>,
        Arc<Mutex<Vec<(ClipboardChangeOrigin, Duration)>>>,
        Arc<AtomicUsize>,
    ) {
        let writes = Arc::new(Mutex::new(Vec::new()));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let origin_values = Arc::new(Mutex::new(Vec::new()));
        let decrypt_calls = Arc::new(AtomicUsize::new(0));

        let usecase = SyncInboundClipboardUseCase::new(
            Arc::new(MockSystemClipboard {
                reads: local_snapshot,
                writes: writes.clone(),
                calls: calls.clone(),
            }),
            Arc::new(MockChangeOrigin {
                calls: calls.clone(),
                values: origin_values.clone(),
            }),
            Arc::new(MockEncryptionSession { ready }),
            Arc::new(MockEncryption {
                decrypt_calls: decrypt_calls.clone(),
            }),
            Arc::new(MockDeviceIdentity {
                id: DeviceId::new(local_device_id),
            }),
        );

        (usecase, writes, calls, origin_values, decrypt_calls)
    }

    #[tokio::test]
    async fn valid_inbound_message_applies_exactly_one_text_plain_snapshot() {
        let (usecase, writes, _, _, _) = build_usecase(
            SystemClipboardSnapshot {
                ts_ms: 0,
                representations: vec![],
            },
            "local-1",
            true,
        );

        usecase
            .execute(build_message("hello inbound", "remote-1"))
            .await
            .expect("execute inbound message");

        let writes = writes.lock().expect("writes lock");
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].representations.len(), 1);
        assert_eq!(
            writes[0].representations[0].mime,
            Some(MimeType::text_plain())
        );
        assert_eq!(writes[0].representations[0].bytes, b"hello inbound");
    }

    #[tokio::test]
    async fn sets_origin_to_remote_push_before_write() {
        let (usecase, _, calls, origin_values, _) = build_usecase(
            SystemClipboardSnapshot {
                ts_ms: 0,
                representations: vec![],
            },
            "local-1",
            true,
        );

        usecase
            .execute(build_message("hello order", "remote-1"))
            .await
            .expect("execute inbound message");

        assert_eq!(
            calls.lock().expect("calls lock").as_slice(),
            ["set_origin", "write_snapshot"]
        );
        let values = origin_values.lock().expect("origin values lock");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, ClipboardChangeOrigin::RemotePush);
        assert_eq!(values[0].1, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn no_op_when_clipboard_already_matches() {
        let (usecase, writes, calls, _, _) =
            build_usecase(build_text_snapshot("same text"), "local-1", true);

        usecase
            .execute(build_message("same text", "remote-1"))
            .await
            .expect("execute inbound message");

        assert_eq!(writes.lock().expect("writes lock").len(), 0);
        assert_eq!(calls.lock().expect("calls lock").len(), 0);
    }

    #[tokio::test]
    async fn ignores_self_origin_messages() {
        let (usecase, writes, calls, _, decrypt_calls) = build_usecase(
            SystemClipboardSnapshot {
                ts_ms: 0,
                representations: vec![],
            },
            "device-self",
            true,
        );

        usecase
            .execute(build_message("self text", "device-self"))
            .await
            .expect("execute inbound message");

        assert_eq!(writes.lock().expect("writes lock").len(), 0);
        assert_eq!(calls.lock().expect("calls lock").len(), 0);
        assert_eq!(decrypt_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn no_op_when_encryption_session_not_ready() {
        let (usecase, writes, calls, _, decrypt_calls) = build_usecase(
            SystemClipboardSnapshot {
                ts_ms: 0,
                representations: vec![],
            },
            "local-1",
            false,
        );

        usecase
            .execute(build_message("not ready", "remote-1"))
            .await
            .expect("execute inbound message");

        assert_eq!(writes.lock().expect("writes lock").len(), 0);
        assert_eq!(calls.lock().expect("calls lock").len(), 0);
        assert_eq!(decrypt_calls.load(Ordering::SeqCst), 0);
    }
}
