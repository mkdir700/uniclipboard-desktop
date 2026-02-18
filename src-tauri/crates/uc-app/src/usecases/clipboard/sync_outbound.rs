use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use futures::executor;
use tracing::{debug, info, info_span, warn, Instrument};
use uuid::Uuid;

use uc_core::network::protocol::ClipboardTextPayloadV1;
use uc_core::network::{ClipboardMessage, ProtocolMessage};
use uc_core::ports::{
    DeviceIdentityPort, EncryptionPort, EncryptionSessionPort, NetworkPort, SettingsPort,
};
use uc_core::security::{aad, model::EncryptionAlgo};
use uc_core::{ClipboardChangeOrigin, SystemClipboardSnapshot};

pub struct SyncOutboundClipboardUseCase {
    network: Arc<dyn NetworkPort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
    encryption: Arc<dyn EncryptionPort>,
    device_identity: Arc<dyn DeviceIdentityPort>,
    settings: Arc<dyn SettingsPort>,
}

impl SyncOutboundClipboardUseCase {
    pub fn new(
        network: Arc<dyn NetworkPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
        encryption: Arc<dyn EncryptionPort>,
        device_identity: Arc<dyn DeviceIdentityPort>,
        settings: Arc<dyn SettingsPort>,
    ) -> Self {
        Self {
            network,
            encryption_session,
            encryption,
            device_identity,
            settings,
        }
    }

    pub fn execute(
        &self,
        snapshot: SystemClipboardSnapshot,
        origin: ClipboardChangeOrigin,
    ) -> Result<()> {
        let span = info_span!(
            "usecase.clipboard.sync_outbound.execute",
            origin = ?origin,
            representation_count = snapshot.representations.len(),
        );

        executor::block_on(self.execute_async(snapshot, origin).instrument(span))
    }

    async fn execute_async(
        &self,
        snapshot: SystemClipboardSnapshot,
        origin: ClipboardChangeOrigin,
    ) -> Result<()> {
        if origin != ClipboardChangeOrigin::LocalCapture {
            debug!(origin = ?origin, "Skipping outbound sync for non-local-capture origin");
            return Ok(());
        }

        if !self.encryption_session.is_ready().await {
            info!(origin = ?origin, "Skipping outbound sync because encryption session is not ready");
            return Ok(());
        }

        let selected_representation = match snapshot.representations.iter().find(|rep| {
            rep.mime
                .as_ref()
                .is_some_and(|mime| is_text_plain_mime(mime.as_str()))
        }) {
            Some(rep) => rep,
            None => {
                debug!(
                    representation_count = snapshot.representations.len(),
                    "Skipping outbound sync because no text/plain representation is available"
                );
                return Ok(());
            }
        };

        let plaintext_text = match std::str::from_utf8(&selected_representation.bytes) {
            Ok(text) => text.to_string(),
            Err(err) => {
                warn!(
                    error = %err,
                    payload_bytes = selected_representation.bytes.len(),
                    "Skipping outbound sync because selected text/plain representation is not valid UTF-8"
                );
                return Ok(());
            }
        };

        let connected_peers = self
            .network
            .get_connected_peers()
            .await
            .context("failed to load connected peers for outbound sync")?;
        if connected_peers.is_empty() {
            debug!("Skipping outbound sync because there are no connected peers");
            return Ok(());
        }

        let message_id = Uuid::new_v4().to_string();
        let payload = ClipboardTextPayloadV1::new(plaintext_text, snapshot.ts_ms);
        let payload_bytes = serde_json::to_vec(&payload)
            .context("failed to serialize clipboard text payload for outbound sync")?;

        let master_key = self
            .encryption_session
            .get_master_key()
            .await
            .map_err(anyhow::Error::from)
            .context("failed to access encryption session master key for outbound sync")?;

        let encrypted_blob = self
            .encryption
            .encrypt_blob(
                &master_key,
                &payload_bytes,
                &aad::for_network_clipboard(&message_id),
                EncryptionAlgo::XChaCha20Poly1305,
            )
            .await
            .map_err(anyhow::Error::from)
            .context("failed to encrypt outbound clipboard payload")?;

        let encrypted_content = serde_json::to_vec(&encrypted_blob)
            .context("failed to serialize encrypted outbound clipboard payload")?;

        let origin_device_id = self.device_identity.current_device_id().to_string();
        let origin_device_name = match self.settings.load().await {
            Ok(settings) => settings
                .general
                .device_name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| "Unknown Device".to_string()),
            Err(err) => {
                warn!(
                    error = %err,
                    "Failed to load settings for outbound sync; using fallback device name"
                );
                "Unknown Device".to_string()
            }
        };

        let clipboard_message = ClipboardMessage {
            id: message_id,
            content_hash: selected_representation.content_hash().to_string(),
            encrypted_content,
            timestamp: Utc::now(),
            origin_device_id,
            origin_device_name,
        };

        let outbound_bytes = ProtocolMessage::Clipboard(clipboard_message)
            .to_bytes()
            .context("failed to serialize outbound protocol clipboard message")?;

        for peer in connected_peers {
            self.network
                .send_clipboard(&peer.peer_id, outbound_bytes.clone())
                .await
                .with_context(|| {
                    format!(
                        "failed to send outbound clipboard message to peer {}",
                        peer.peer_id
                    )
                })?;
        }

        info!("Outbound clipboard sync sent to connected peers");
        Ok(())
    }
}

fn is_text_plain_mime(mime: &str) -> bool {
    let normalized = mime.trim();
    normalized.eq_ignore_ascii_case("text/plain")
        || normalized.to_ascii_lowercase().starts_with("text/plain;")
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::Utc;
    use tokio::sync::mpsc;
    use uc_core::ids::{FormatId, RepresentationId};
    use uc_core::network::protocol::ClipboardTextPayloadV1;
    use uc_core::network::{
        ClipboardMessage, ConnectedPeer, DiscoveredPeer, NetworkEvent, PairingMessage,
        ProtocolMessage,
    };
    use uc_core::security::model::{
        EncryptedBlob, EncryptionAlgo, EncryptionError, EncryptionFormatVersion, KdfParams, Kek,
        MasterKey, Passphrase,
    };
    use uc_core::settings::model::Settings;
    use uc_core::{DeviceId, MimeType, ObservedClipboardRepresentation, SystemClipboardSnapshot};

    struct TestNetwork {
        connected_peers: Vec<ConnectedPeer>,
        send_calls: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
        get_connected_peers_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl NetworkPort for TestNetwork {
        async fn send_clipboard(
            &self,
            peer_id: &str,
            encrypted_data: Vec<u8>,
        ) -> anyhow::Result<()> {
            self.send_calls
                .lock()
                .expect("send calls lock")
                .push((peer_id.to_string(), encrypted_data));
            Ok(())
        }

        async fn broadcast_clipboard(&self, _encrypted_data: Vec<u8>) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_clipboard(&self) -> anyhow::Result<mpsc::Receiver<ClipboardMessage>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }

        async fn get_discovered_peers(&self) -> anyhow::Result<Vec<DiscoveredPeer>> {
            Ok(Vec::new())
        }

        async fn get_connected_peers(&self) -> anyhow::Result<Vec<ConnectedPeer>> {
            self.get_connected_peers_calls
                .fetch_add(1, Ordering::SeqCst);
            Ok(self.connected_peers.clone())
        }

        fn local_peer_id(&self) -> String {
            "peer-local".to_string()
        }

        async fn announce_device_name(&self, _device_name: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn open_pairing_session(
            &self,
            _peer_id: String,
            _session_id: String,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn send_pairing_on_session(
            &self,
            _session_id: String,
            _message: PairingMessage,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn close_pairing_session(
            &self,
            _session_id: String,
            _reason: Option<String>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn unpair_device(&self, _peer_id: String) -> anyhow::Result<()> {
            Ok(())
        }

        async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<NetworkEvent>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }
    }

    struct TestEncryptionSession {
        ready: bool,
    }

    #[async_trait]
    impl EncryptionSessionPort for TestEncryptionSession {
        async fn is_ready(&self) -> bool {
            self.ready
        }

        async fn get_master_key(&self) -> std::result::Result<MasterKey, EncryptionError> {
            Ok(MasterKey([7; 32]))
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

    struct TestEncryption {
        encrypt_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl EncryptionPort for TestEncryption {
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
            plaintext: &[u8],
            _aad: &[u8],
            _aead: EncryptionAlgo,
        ) -> std::result::Result<EncryptedBlob, EncryptionError> {
            self.encrypt_calls.fetch_add(1, Ordering::SeqCst);
            Ok(EncryptedBlob {
                version: EncryptionFormatVersion::V1,
                aead: EncryptionAlgo::XChaCha20Poly1305,
                nonce: vec![1; 24],
                ciphertext: plaintext.to_vec(),
                aad_fingerprint: None,
            })
        }

        async fn decrypt_blob(
            &self,
            _master_key: &MasterKey,
            _encrypted: &EncryptedBlob,
            _aad: &[u8],
        ) -> std::result::Result<Vec<u8>, EncryptionError> {
            Err(EncryptionError::CorruptedBlob)
        }
    }

    struct TestDeviceIdentity;

    impl DeviceIdentityPort for TestDeviceIdentity {
        fn current_device_id(&self) -> DeviceId {
            DeviceId::new("device-1")
        }
    }

    struct TestSettings {
        settings: Settings,
    }

    #[async_trait]
    impl SettingsPort for TestSettings {
        async fn load(&self) -> anyhow::Result<Settings> {
            Ok(self.settings.clone())
        }

        async fn save(&self, _settings: &Settings) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn build_snapshot() -> SystemClipboardSnapshot {
        SystemClipboardSnapshot {
            ts_ms: 1_713_000_000_000,
            representations: vec![ObservedClipboardRepresentation {
                id: RepresentationId::new(),
                format_id: FormatId::from("public.utf8-plain-text"),
                mime: Some(MimeType::text_plain()),
                bytes: b"hello world".to_vec(),
            }],
        }
    }

    fn build_usecase(
        connected_peers: Vec<ConnectedPeer>,
        encryption_ready: bool,
    ) -> (
        SyncOutboundClipboardUseCase,
        Arc<Mutex<Vec<(String, Vec<u8>)>>>,
        Arc<AtomicUsize>,
        Arc<AtomicUsize>,
    ) {
        let send_calls = Arc::new(Mutex::new(Vec::new()));
        let get_connected_peers_calls = Arc::new(AtomicUsize::new(0));
        let encrypt_calls = Arc::new(AtomicUsize::new(0));

        let usecase = SyncOutboundClipboardUseCase::new(
            Arc::new(TestNetwork {
                connected_peers,
                send_calls: send_calls.clone(),
                get_connected_peers_calls: get_connected_peers_calls.clone(),
            }),
            Arc::new(TestEncryptionSession {
                ready: encryption_ready,
            }),
            Arc::new(TestEncryption {
                encrypt_calls: encrypt_calls.clone(),
            }),
            Arc::new(TestDeviceIdentity),
            Arc::new(TestSettings {
                settings: Settings::default(),
            }),
        );

        (
            usecase,
            send_calls,
            get_connected_peers_calls,
            encrypt_calls,
        )
    }

    #[test]
    fn sends_exactly_once_for_local_capture_when_peer_exists() {
        let (usecase, send_calls, _, _) = build_usecase(
            vec![ConnectedPeer {
                peer_id: "peer-1".to_string(),
                device_name: "Desk".to_string(),
                connected_at: Utc::now(),
            }],
            true,
        );

        usecase
            .execute(build_snapshot(), ClipboardChangeOrigin::LocalCapture)
            .expect("execute local capture");

        assert_eq!(send_calls.lock().expect("send calls lock").len(), 1);
    }

    #[test]
    fn does_not_send_for_remote_push_or_local_restore() {
        let (usecase, send_calls, _, _) = build_usecase(
            vec![ConnectedPeer {
                peer_id: "peer-1".to_string(),
                device_name: "Desk".to_string(),
                connected_at: Utc::now(),
            }],
            true,
        );

        usecase
            .execute(build_snapshot(), ClipboardChangeOrigin::RemotePush)
            .expect("remote push should no-op");
        usecase
            .execute(build_snapshot(), ClipboardChangeOrigin::LocalRestore)
            .expect("local restore should no-op");

        assert_eq!(send_calls.lock().expect("send calls lock").len(), 0);
    }

    #[test]
    fn no_op_when_encryption_session_not_ready() {
        let (usecase, send_calls, get_connected_peers_calls, encrypt_calls) = build_usecase(
            vec![ConnectedPeer {
                peer_id: "peer-1".to_string(),
                device_name: "Desk".to_string(),
                connected_at: Utc::now(),
            }],
            false,
        );

        usecase
            .execute(build_snapshot(), ClipboardChangeOrigin::LocalCapture)
            .expect("execute should no-op");

        assert_eq!(send_calls.lock().expect("send calls lock").len(), 0);
        assert_eq!(get_connected_peers_calls.load(Ordering::SeqCst), 0);
        assert_eq!(encrypt_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn outbound_bytes_decode_as_protocol_message_clipboard() {
        let (usecase, send_calls, _, _) = build_usecase(
            vec![ConnectedPeer {
                peer_id: "peer-1".to_string(),
                device_name: "Desk".to_string(),
                connected_at: Utc::now(),
            }],
            true,
        );

        usecase
            .execute(build_snapshot(), ClipboardChangeOrigin::LocalCapture)
            .expect("execute local capture");

        let calls = send_calls.lock().expect("send calls lock");
        let (_, outbound_bytes) = calls.first().expect("one outbound send");
        let decoded = ProtocolMessage::from_bytes(outbound_bytes).expect("decode protocol message");

        match decoded {
            ProtocolMessage::Clipboard(message) => {
                let encrypted_blob: EncryptedBlob =
                    serde_json::from_slice(&message.encrypted_content)
                        .expect("decode encrypted blob");
                let payload: ClipboardTextPayloadV1 =
                    serde_json::from_slice(&encrypted_blob.ciphertext).expect("decode payload");
                assert_eq!(payload.text, "hello world");
            }
            _ => panic!("expected ProtocolMessage::Clipboard"),
        }
    }
}
