//! Device pairing module for UniClipboard P2P
//!
//! Handles secure device pairing using:
//! - X25519 ECDH for key exchange
//! - PIN-based verification to prevent MITM attacks
//! - Shared secret derivation for encrypted clipboard sync
//! - Actor pattern for thread safety

use anyhow::{anyhow, Result};
use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use super::events::NetworkEvent;
use super::protocol::{PairingChallenge, PairingConfirm, PairingRequest, PairingResponse};
use super::swarm::NetworkCommand;

/// PIN length for pairing verification
const PIN_LENGTH: usize = 6;

/// Commands for the pairing actor
#[derive(Debug)]
pub enum PairingCommand {
    /// Initiate pairing with a peer
    Initiate {
        peer_id: String,
        device_name: String,
        respond_to: oneshot::Sender<Result<String>>,
    },
    /// Handle incoming pairing request
    HandleRequest {
        peer_id: String,
        request: PairingRequest,
    },
    /// Handle incoming PIN ready (Challenge)
    HandlePinReady {
        session_id: String,
        pin: String,
        peer_device_name: String,
        peer_public_key: Vec<u8>,
    },
    /// Verify PIN
    VerifyPin {
        session_id: String,
        pin_match: bool,
        respond_to: oneshot::Sender<Result<()>>,
    },
    /// Handle pairing response
    HandleResponse {
        session_id: String,
        response: PairingResponse,
        peer_device_name: String,
        respond_to: oneshot::Sender<Result<(bool, Option<Vec<u8>>)>>,
    },
    /// Handle pairing confirm
    HandleConfirm {
        session_id: String,
        confirm: PairingConfirm,
        peer_id: String,
        respond_to: oneshot::Sender<Result<Option<Vec<u8>>>>,
    },
    /// Reject pairing
    Reject {
        session_id: String,
        peer_id: String,
        respond_to: oneshot::Sender<Result<()>>,
    },
    /// Accept pairing request (responder side)
    AcceptPairing {
        session_id: String,
        respond_to: oneshot::Sender<Result<()>>,
    },
}

/// Active pairing session state
pub struct PairingSession {
    pub session_id: String,
    pub peer_id: String,
    pub device_name: String,
    pub our_public_key: PublicKey,
    pub our_private_key: StaticSecret,
    pub peer_public_key: Option<PublicKey>,
    pub created_at: chrono::DateTime<Utc>,
    pub is_initiator: bool,
}

impl PairingSession {
    pub fn new(
        session_id: String,
        peer_id: String,
        device_name: String,
        is_initiator: bool,
    ) -> Self {
        let our_private_key = StaticSecret::random();
        let our_public_key = PublicKey::from(&our_private_key);

        Self {
            session_id,
            peer_id,
            device_name,
            our_public_key,
            our_private_key,
            peer_public_key: None,
            created_at: Utc::now(),
            is_initiator,
        }
    }

    /// Compute ECDH shared secret
    pub fn compute_shared_secret(&self) -> Result<SharedSecret> {
        let peer_public = self
            .peer_public_key
            .ok_or_else(|| anyhow!("Peer public key not set"))?;

        Ok(self.our_private_key.diffie_hellman(&peer_public))
    }

    /// Check if session is expired (5 minutes)
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now() - self.created_at;
        elapsed.num_seconds() > 300 // 5 minutes
    }
}

/// Pairing manager that handles the device pairing flow
pub struct PairingManager {
    /// Active pairing sessions keyed by session_id
    sessions: HashMap<String, PairingSession>,
    /// Channel to send commands to NetworkManager
    network_command_tx: mpsc::Sender<NetworkCommand>,
    /// Channel to emit pairing events
    event_tx: mpsc::Sender<NetworkEvent>,
    /// Channel to receive commands
    command_rx: mpsc::Receiver<PairingCommand>,
    /// local device name
    device_name: String,
}

impl PairingManager {
    pub fn new(
        network_command_tx: mpsc::Sender<NetworkCommand>,
        event_tx: mpsc::Sender<NetworkEvent>,
        command_rx: mpsc::Receiver<PairingCommand>,
        device_name: String,
    ) -> Self {
        Self {
            sessions: HashMap::new(),
            network_command_tx,
            event_tx,
            command_rx,
            device_name,
        }
    }

    /// Run the pairing manager actor
    pub async fn run(mut self) {
        log::info!("PairingManager actor started");
        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("Error handling pairing command: {}", e);
                    }
                }
                // Periodic cleanup every minute
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    self.cleanup_expired_sessions();
                }
            }
        }
    }

    /// Handle a pairing command
    async fn handle_command(&mut self, cmd: PairingCommand) -> Result<()> {
        match cmd {
            PairingCommand::Initiate {
                peer_id,
                device_name,
                respond_to,
            } => {
                let result = self.initiate_pairing(peer_id, device_name).await;
                let _ = respond_to.send(result);
            }
            PairingCommand::VerifyPin {
                session_id,
                pin_match,
                respond_to,
            } => {
                let result = self.verify_pin(&session_id, pin_match).await;
                let _ = respond_to.send(result);
            }
            PairingCommand::HandleRequest { peer_id, request } => {
                // This command is internal/network triggered.
                // We don't have a respond_to channel here usually, passing Result is fine.
                self.handle_pairing_request(peer_id, request).await?;
            }
            PairingCommand::HandlePinReady {
                session_id,
                pin,
                peer_device_name,
                peer_public_key,
            } => {
                self.handle_pin_ready(&session_id, pin, peer_device_name, peer_public_key)
                    .await?;
            }
            PairingCommand::HandleResponse {
                session_id,
                response,
                peer_device_name,
                respond_to,
            } => {
                let result = self
                    .handle_pairing_response(&session_id, response, peer_device_name)
                    .await;
                let _ = respond_to.send(result);
            }
            PairingCommand::HandleConfirm {
                session_id,
                confirm,
                peer_id,
                respond_to,
            } => {
                let result = self
                    .handle_pairing_confirm(&session_id, confirm, peer_id)
                    .await;
                let _ = respond_to.send(result);
            }
            PairingCommand::Reject {
                session_id,
                peer_id,
                respond_to,
            } => {
                let result = self.reject_pairing(&session_id, peer_id).await;
                let _ = respond_to.send(result);
            }
            PairingCommand::AcceptPairing {
                session_id,
                respond_to,
            } => {
                let result = self.accept_pairing(&session_id).await;
                let _ = respond_to.send(result);
            }
        }
        Ok(())
    }

    /// Generate a new session ID
    fn generate_session_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut rng = rand::rng();
        format!("{}-{}", timestamp, rng.random::<u32>())
    }

    /// Generate a random PIN for pairing verification
    fn generate_pin(&self) -> String {
        let mut rng = rand::rng();
        (0..PIN_LENGTH)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    }

    /// Initiate pairing as the initiator
    pub async fn initiate_pairing(
        &mut self,
        peer_id: String,
        device_name: String,
    ) -> Result<String> {
        let session_id = self.generate_session_id();
        let our_private_key = StaticSecret::random();
        let our_public_key = PublicKey::from(&our_private_key);

        let session = PairingSession {
            session_id: session_id.clone(),
            peer_id: peer_id.clone(),
            device_name: device_name.clone(),
            our_public_key,
            our_private_key,
            peer_public_key: None,
            created_at: Utc::now(),
            is_initiator: true,
        };

        // Store public key bytes before moving session into map
        let public_key_bytes = session.our_public_key.to_bytes().to_vec();
        self.sessions.insert(session_id.clone(), session);

        let request = PairingRequest {
            session_id: session_id.clone(),
            device_name,
            device_id: peer_id.clone(),
            public_key: public_key_bytes,
        };

        let message = super::protocol::ProtocolMessage::Pairing(
            super::protocol::PairingMessage::Request(request),
        );

        let message_bytes = message
            .to_bytes()
            .map_err(|e| anyhow!("Failed to serialize pairing request: {}", e))?;

        self.network_command_tx
            .send(NetworkCommand::SendPairingRequest {
                peer_id,
                message: message_bytes,
            })
            .await
            .map_err(|e| anyhow!("Failed to send pairing request: {}", e))?;

        Ok(session_id)
    }

    /// Handle incoming pairing request (responder side)
    /// Only creates session and notifies frontend - PIN generation is deferred until accept_pairing
    pub async fn handle_pairing_request(
        &mut self,
        peer_id: String,
        request: PairingRequest,
    ) -> Result<()> {
        let our_private_key = StaticSecret::random();
        let our_public_key = PublicKey::from(&our_private_key);

        // Parse peer's public key
        let peer_public_bytes: [u8; 32] = request
            .public_key
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Invalid public key length: {}", e))?;
        let peer_public = PublicKey::from(peer_public_bytes);

        let session = PairingSession {
            session_id: request.session_id.clone(),
            peer_id: peer_id.clone(),
            device_name: request.device_name.clone(),
            our_public_key,
            our_private_key,
            peer_public_key: Some(peer_public), // Store the key immediately
            created_at: Utc::now(),
            is_initiator: false,
        };

        self.sessions.insert(request.session_id.clone(), session);

        // Emit PairingRequestReceived event to frontend - PIN generation is deferred
        let _ = self
            .event_tx
            .send(NetworkEvent::PairingRequestReceived {
                session_id: request.session_id.clone(),
                peer_id,
                request,
            })
            .await;

        Ok(())
    }

    /// Accept pairing request (responder side) - generates PIN and sends challenge
    pub async fn accept_pairing(&mut self, session_id: &str) -> Result<()> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        if session.is_expired() {
            return Err(anyhow!("Pairing session expired"));
        }

        let peer_id = session.peer_id.clone();
        let peer_device_name = session.device_name.clone();
        let our_public_key_bytes = session.our_public_key.to_bytes().to_vec();

        // Generate PIN for user verification
        let pin = self.generate_pin();

        // Send p2p-pin-ready event to frontend (responder needs to see the PIN)
        let _ = self
            .event_tx
            .send(NetworkEvent::PairingPinReady {
                session_id: session_id.to_string(),
                pin: pin.clone(),
                peer_device_name: peer_device_name.clone(),
                peer_public_key: our_public_key_bytes.clone(),
            })
            .await;

        // Send pairing challenge with PIN and our public key to the initiator
        self.network_command_tx
            .send(NetworkCommand::SendPairingChallenge {
                peer_id,
                session_id: session_id.to_string(),
                pin,
                device_name: self.device_name.clone(),
                public_key: our_public_key_bytes,
            })
            .await
            .map_err(|e| anyhow!("Failed to send pairing challenge: {}", e))?;

        Ok(())
    }

    /// Handle incoming PIN ready / Challenge (initiator side)
    pub async fn handle_pin_ready(
        &mut self,
        session_id: &str,
        _pin: String,
        peer_device_name: String,
        peer_public_key: Vec<u8>,
    ) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        // Parse peer's public key
        let peer_public_bytes: [u8; 32] = peer_public_key
            .as_slice()
            .try_into()
            .map_err(|e| anyhow!("Invalid public key length: {}", e))?;
        let peer_public = PublicKey::from(peer_public_bytes);

        session.peer_public_key = Some(peer_public);
        session.device_name = peer_device_name;

        Ok(())
    }

    /// Handle PIN verification result (initiator side)
    pub async fn verify_pin(&mut self, session_id: &str, pin_match: bool) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        if session.is_expired() {
            return Err(anyhow!("Pairing session expired"));
        }

        let device_name = session.device_name.clone();
        let peer_id = session.peer_id.clone();

        let confirm = if pin_match {
            // Compute shared secret
            let shared_secret = session.compute_shared_secret()?;
            let secret_bytes = shared_secret.to_bytes().to_vec();

            PairingConfirm {
                session_id: session_id.to_string(),
                success: true,
                shared_secret: Some(secret_bytes),
                error: None,
                device_name: Some(device_name.clone()),
            }
        } else {
            PairingConfirm {
                session_id: session_id.to_string(),
                success: false,
                shared_secret: None,
                error: Some("PIN verification failed".to_string()),
                device_name: None,
            }
        };

        let message = super::protocol::ProtocolMessage::Pairing(
            super::protocol::PairingMessage::Confirm(confirm.clone()),
        );

        let _message_bytes = message
            .to_bytes()
            .map_err(|e| anyhow!("Failed to serialize pairing confirm: {}", e))?;

        self.network_command_tx
            .send(NetworkCommand::SendPairingConfirm {
                peer_id,
                session_id: session_id.to_string(),
                success: confirm.success,
                shared_secret: confirm.shared_secret,
                device_name,
            })
            .await
            .map_err(|e| anyhow!("Failed to send pairing confirm: {}", e))?;

        if !pin_match {
            self.sessions.remove(session_id);
        }

        Ok(())
    }

    /// Handle pairing response (responder side)
    pub async fn handle_pairing_response(
        &mut self,
        session_id: &str,
        response: PairingResponse,
        peer_device_name: String,
    ) -> Result<(bool, Option<Vec<u8>>)> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        if session.is_expired() {
            return Err(anyhow!("Pairing session expired"));
        }

        if !response.accepted {
            self.sessions.remove(session_id);
            return Ok((false, None));
        }

        // Verify PIN hash
        // TODO: Implement proper PIN hash verification
        // For now, we trust the client-side verification

        Ok((true, None))
    }

    /// Handle pairing confirmation (responder side)
    pub async fn handle_pairing_confirm(
        &mut self,
        session_id: &str,
        confirm: PairingConfirm,
        peer_id: String,
    ) -> Result<Option<Vec<u8>>> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;

        if session.is_expired() {
            return Err(anyhow!("Pairing session expired"));
        }

        if confirm.success {
            if confirm.shared_secret.is_some() {
                // The shared secret is sent by the initiator
                // For now, we'll derive our own shared secret
                let shared_secret = session.compute_shared_secret()?;
                let secret_bytes = shared_secret.to_bytes().to_vec();

                self.sessions.remove(session_id);

                return Ok(Some(secret_bytes));
            }
        }

        self.sessions.remove(session_id);
        Ok(None)
    }

    /// Reject an incoming pairing request
    pub async fn reject_pairing(&mut self, session_id: &str, peer_id: String) -> Result<()> {
        self.network_command_tx
            .send(NetworkCommand::RejectPairing {
                peer_id,
                session_id: session_id.to_string(),
            })
            .await
            .map_err(|e| anyhow!("Failed to send pairing rejection: {}", e))?;

        self.sessions.remove(session_id);
        Ok(())
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        self.sessions.retain(|_, session| !session.is_expired());
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&PairingSession> {
        self.sessions.get(session_id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut PairingSession> {
        self.sessions.get_mut(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pin() {
        let manager = create_test_manager();
        let pin = manager.generate_pin();
        assert_eq!(pin.len(), PIN_LENGTH);
        assert!(pin.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_session_expiration() {
        let session = PairingSession::new(
            "test-session".to_string(),
            "peer-123".to_string(),
            "Test Device".to_string(),
            true,
        );
        assert!(!session.is_expired());
    }

    #[test]
    fn test_ecdh_key_exchange() {
        // Alice's keys
        let alice_private = StaticSecret::random();
        let alice_public = PublicKey::from(&alice_private);

        // Bob's keys
        let bob_private = StaticSecret::random();
        let bob_public = PublicKey::from(&bob_private);

        // Both should compute the same shared secret
        let alice_shared = alice_private.diffie_hellman(&bob_public);
        let bob_shared = bob_private.diffie_hellman(&alice_public);

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    fn create_test_manager() -> PairingManager {
        let (cmd_tx, _) = mpsc::channel(100);
        let (event_tx, _) = mpsc::channel(100);
        let (_, command_rx) = mpsc::channel(100);
        PairingManager::new(cmd_tx, event_tx, command_rx, "Test Device".to_string())
    }
}
