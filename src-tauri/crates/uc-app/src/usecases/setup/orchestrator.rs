//! Setup orchestrator.
//!
//! This module coordinates the setup state machine and side effects.

use std::sync::Arc;

use tracing::{error, info_span, Instrument};

use uc_core::{
    security::model::Passphrase,
    setup::{SetupAction, SetupEvent, SetupState, SetupStateMachine},
};

use crate::usecases::initialize_encryption::InitializeEncryptionError;
use crate::usecases::setup::context::SetupContext;
use crate::usecases::CompleteOnboarding;
use crate::usecases::InitializeEncryption;

/// Errors produced by the setup orchestrator.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("initialize encryption failed: {0}")]
    InitializeEncryption(#[from] InitializeEncryptionError),
    #[error("complete onboarding failed: {0}")]
    CompleteOnboarding(#[from] anyhow::Error),
    #[error("setup action not implemented: {0}")]
    ActionNotImplemented(&'static str),
}

/// Orchestrator that drives setup state and side effects.
pub struct SetupOrchestrator {
    context: Arc<SetupContext>,

    // 能力型 use cases (依赖注入)
    initialize_encryption: Arc<InitializeEncryption>,
    complete_onboarding: Arc<CompleteOnboarding>,
}

impl SetupOrchestrator {
    pub fn new(
        initialize_encryption: Arc<InitializeEncryption>,
        complete_onboarding: Arc<CompleteOnboarding>,
    ) -> Self {
        Self {
            context: SetupContext::default().arc(),
            initialize_encryption,
            complete_onboarding,
        }
    }

    pub async fn new_space(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::StartNewSpace;
        self.dispatch(event).await
    }

    pub async fn join_space(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::StartJoinSpace;
        self.dispatch(event).await
    }

    pub async fn select_device(&self, peer_id: String) -> Result<SetupState, SetupError> {
        let event = SetupEvent::SelectPeer { peer_id };
        self.dispatch(event).await
    }

    pub async fn submit_passphrase(
        &self,
        pass1: String,
        pass2: String,
    ) -> Result<SetupState, SetupError> {
        let event = SetupEvent::SubmitCreatePassphrase { pass1, pass2 };
        self.dispatch(event).await
    }

    pub async fn verify_passphrase(&self, passphrase: String) -> Result<SetupState, SetupError> {
        let event = SetupEvent::SubmitJoinPassphrase { passphrase };
        self.dispatch(event).await
    }

    pub async fn cancel_setup(&self) -> Result<SetupState, SetupError> {
        let event = SetupEvent::UserCancel;
        self.dispatch(event).await
    }

    pub async fn get_state(&self) -> SetupState {
        self.context.get_state().await
    }

    async fn dispatch(&self, event: SetupEvent) -> Result<SetupState, SetupError> {
        // Acquire dispatch lock to serialize concurrent dispatch calls.
        // This prevents race conditions where multiple calls read the same state
        // and execute duplicate actions.
        let _dispatch_guard = self.context.acquire_dispatch_lock().await;

        let span = info_span!("usecase.setup_orchestrator.dispatch", event = ?event);
        async {
            let current = self.context.get_state().await;
            let (next, actions) = SetupStateMachine::transition(current, event);
            self.execute_actions(actions).await?;
            self.context.set_state(next.clone()).await;
            Ok(next)
        }
        .instrument(span)
        .await
    }

    async fn execute_actions(&self, actions: Vec<SetupAction>) -> Result<(), SetupError> {
        for action in actions {
            match action {
                SetupAction::CreateEncryptedSpace { passphrase } => {
                    self.initialize_encryption
                        .execute(Passphrase(passphrase))
                        .await?;
                }
                SetupAction::MarkSetupComplete => {
                    self.complete_onboarding.execute().await?;
                }
                SetupAction::ScanPeers => {
                    error!("Setup action ScanPeers is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("ScanPeers"));
                }
                SetupAction::VerifyPassphraseWithPeer { .. } => {
                    error!("Setup action VerifyPassphraseWithPeer is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("VerifyPassphraseWithPeer"));
                }
                SetupAction::StartPairing { .. } => {
                    error!("Setup action StartPairing is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("StartPairing"));
                }
                SetupAction::ConfirmPairing { .. } => {
                    error!("Setup action ConfirmPairing is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("ConfirmPairing"));
                }
                SetupAction::CancelPairing { .. } => {
                    error!("Setup action CancelPairing is not implemented yet");
                    return Err(SetupError::ActionNotImplemented("CancelPairing"));
                }
            }
        }

        Ok(())
    }
}
