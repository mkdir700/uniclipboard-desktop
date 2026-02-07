use async_trait::async_trait;
use tokio::sync::mpsc;

use uc_core::network::pairing_state_machine::FailureReason;
use uc_core::security::model::KeySlotFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingDomainEvent {
    KeyslotReceived {
        session_id: String,
        peer_id: String,
        keyslot_file: KeySlotFile,
        challenge: Vec<u8>,
    },
    PairingVerificationRequired {
        session_id: String,
        peer_id: String,
        short_code: String,
        local_fingerprint: String,
        peer_fingerprint: String,
    },
    PairingSucceeded {
        session_id: String,
        peer_id: String,
    },
    PairingFailed {
        session_id: String,
        peer_id: String,
        reason: FailureReason,
    },
}

#[async_trait]
pub trait PairingEventPort: Send + Sync {
    async fn subscribe(&self) -> anyhow::Result<mpsc::Receiver<PairingDomainEvent>>;
}
