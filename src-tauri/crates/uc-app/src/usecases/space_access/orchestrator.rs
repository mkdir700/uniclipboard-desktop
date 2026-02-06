//! Space access orchestrator.
//!
//! Coordinates space access state machine and side effects.

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info_span, warn, Instrument};

use uc_core::ids::SpaceId;
use uc_core::network::SessionId;
use uc_core::security::space_access::action::SpaceAccessAction;
use uc_core::security::space_access::event::SpaceAccessEvent;
use uc_core::security::space_access::state::SpaceAccessState;
use uc_core::security::space_access::state_machine::SpaceAccessStateMachine;
use uc_core::SessionId as CoreSessionId;

use super::context::{SpaceAccessContext, SpaceAccessOffer};
use super::executor::SpaceAccessExecutor;

/// Errors produced by space access orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum SpaceAccessError {
    #[error("space access action not implemented: {0}")]
    ActionNotImplemented(&'static str),
    #[error("space access missing pairing session id")]
    MissingPairingSessionId,
    #[error("space access crypto failed: {0}")]
    Crypto(#[from] anyhow::Error),
    #[error("space access timer failed: {0}")]
    Timer(#[source] anyhow::Error),
}

/// Orchestrator that drives space access state and side effects.
pub struct SpaceAccessOrchestrator {
    context: Arc<Mutex<SpaceAccessContext>>,
    state: Arc<Mutex<SpaceAccessState>>,
    dispatch_lock: Arc<Mutex<()>>,
}

impl SpaceAccessOrchestrator {
    pub fn new() -> Self {
        Self::with_context(SpaceAccessContext::default())
    }

    pub fn with_context(context: SpaceAccessContext) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
            state: Arc::new(Mutex::new(SpaceAccessState::Idle)),
            dispatch_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn initialize_new_space(
        &self,
        executor: &mut SpaceAccessExecutor<'_>,
        pairing_session_id: SessionId,
        space_id: SpaceId,
        ttl_secs: u64,
    ) -> Result<SpaceAccessState, SpaceAccessError> {
        let event = SpaceAccessEvent::SponsorAuthorizationRequested {
            pairing_session_id: pairing_session_id.clone(),
            space_id,
            ttl_secs,
        };
        self.dispatch(executor, event, Some(pairing_session_id))
            .await
    }

    pub async fn get_state(&self) -> SpaceAccessState {
        self.state.lock().await.clone()
    }

    async fn dispatch(
        &self,
        executor: &mut SpaceAccessExecutor<'_>,
        event: SpaceAccessEvent,
        pairing_session_id: Option<SessionId>,
    ) -> Result<SpaceAccessState, SpaceAccessError> {
        let _dispatch_guard = self.dispatch_lock.lock().await;

        let span = info_span!("usecase.space_access_orchestrator.dispatch", event = ?event);
        async {
            let current = self.state.lock().await.clone();
            let (next, actions) = SpaceAccessStateMachine::transition(current, event);
            self.execute_actions(executor, pairing_session_id.as_ref(), actions)
                .await?;
            let mut guard = self.state.lock().await;
            *guard = next.clone();
            Ok(next)
        }
        .instrument(span)
        .await
    }

    async fn execute_actions(
        &self,
        executor: &mut SpaceAccessExecutor<'_>,
        pairing_session_id: Option<&SessionId>,
        actions: Vec<SpaceAccessAction>,
    ) -> Result<(), SpaceAccessError> {
        for action in actions {
            match action {
                SpaceAccessAction::RequestOfferPreparation {
                    pairing_session_id,
                    space_id,
                    expires_at: _,
                } => {
                    let keyslot = executor.crypto.export_keyslot_blob(&space_id).await?;
                    let nonce = executor.crypto.generate_nonce32().await;
                    let offer = SpaceAccessOffer {
                        space_id: space_id.clone(),
                        keyslot,
                        nonce,
                    };
                    let mut context = self.context.lock().await;
                    context.prepared_offer = Some(offer);
                    let _ = pairing_session_id;
                }
                SpaceAccessAction::SendOffer => {
                    warn!("space access send_offer is not wired yet");
                }
                SpaceAccessAction::StartTimer { ttl_secs } => {
                    let session_id =
                        pairing_session_id.ok_or(SpaceAccessError::MissingPairingSessionId)?;
                    let session_id = CoreSessionId::from(session_id.as_str());
                    executor
                        .timer
                        .start(&session_id, ttl_secs)
                        .await
                        .map_err(SpaceAccessError::Timer)?;
                }
                SpaceAccessAction::StopTimer => {
                    let session_id =
                        pairing_session_id.ok_or(SpaceAccessError::MissingPairingSessionId)?;
                    let session_id = CoreSessionId::from(session_id.as_str());
                    executor
                        .timer
                        .stop(&session_id)
                        .await
                        .map_err(SpaceAccessError::Timer)?;
                }
                SpaceAccessAction::RequestSpaceKeyDerivation { .. } => {
                    error!("space access action RequestSpaceKeyDerivation is not implemented");
                    return Err(SpaceAccessError::ActionNotImplemented(
                        "RequestSpaceKeyDerivation",
                    ));
                }
                SpaceAccessAction::SendProof => {
                    error!("space access action SendProof is not implemented");
                    return Err(SpaceAccessError::ActionNotImplemented("SendProof"));
                }
                SpaceAccessAction::SendResult => {
                    error!("space access action SendResult is not implemented");
                    return Err(SpaceAccessError::ActionNotImplemented("SendResult"));
                }
                SpaceAccessAction::PersistJoinerAccess { .. } => {
                    error!("space access action PersistJoinerAccess is not implemented");
                    return Err(SpaceAccessError::ActionNotImplemented(
                        "PersistJoinerAccess",
                    ));
                }
                SpaceAccessAction::PersistSponsorAccess { .. } => {
                    error!("space access action PersistSponsorAccess is not implemented");
                    return Err(SpaceAccessError::ActionNotImplemented(
                        "PersistSponsorAccess",
                    ));
                }
            }
        }

        Ok(())
    }
}
