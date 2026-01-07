use serde::{Deserialize, Serialize};

/// Device pairing state machine
///
/// Design principle: This is a pure type state machine with only state
/// definitions and transition validation logic. Runtime behaviors like
/// timeouts and automatic state transitions are handled by the application layer (uc-app).
///
/// State transitions:
/// ```text
/// Responder（接收方）:
///   Idle
///    │ IncomingRequest
///    ▼
///   IncomingRequest
///    ├── UserAccepted ───────────────► PendingResponse
///    │                                  │
///    │                                  ├── ResponseReceived{success=true}
///    │                                  │        ▼
///    │                                  │    WaitingConfirm
///    │                                  │        │
///    │                                  │        ├── ConfirmReceived{success=true}  ─► Paired
///    │                                  │        └── ConfirmReceived{success=false} ─► Failed
///    │                                  │
///    │                                  └── ResponseReceived{success=false} ─────────► Failed
///    │
///    └── UserRejected ───────────────────────────────────────────────► Rejected
///
///
/// Initiator（发起方）:
///   Requesting
///    │ ChallengeReceived
///    ▼
///   PendingChallenge
///    ├── PinVerified{success=true}  ───► Verifying
///    │                                   │
///    │                                   ├── ConfirmReceived{success=true}  ─► Paired
///    │                                   └── ConfirmReceived{success=false} ─► Failed
///    │
///    └── PinVerified{success=false} ───► Failed
///
/// Global（全局规则）:
///   任意 Active 状态 + Timeout  ──────────────────────────────────────► Expired
///   任意 状态 + UserRejected     ──────────────────────────────────────► Rejected
///
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairingState {
    /// No pairing in progress
    Idle,

    /// Initiator: Sent pairing request, waiting for challenge
    Requesting,

    /// Responder: Received pairing request, waiting for user to accept
    IncomingRequest,

    /// Responder: User accepted, generated PIN, sent challenge
    PendingResponse,

    /// Initiator: Received challenge with PIN, waiting for user verification
    PendingChallenge,

    /// Initiator: User verified PIN, sent response
    Verifying,

    /// Both: Waiting for final confirmation message
    WaitingConfirm,

    /// Pairing completed successfully
    Paired,

    /// Pairing failed (wrong PIN, timeout, etc.)
    Failed,

    /// Pairing session expired
    Expired,

    /// Pairing rejected by user
    Rejected,
}

impl PairingState {
    /// Check if this is a terminal state (no more transitions possible)
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Paired | Self::Failed | Self::Expired | Self::Rejected
        )
    }

    /// Check if this is an active pairing state
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Requesting
                | Self::IncomingRequest
                | Self::PendingResponse
                | Self::PendingChallenge
                | Self::Verifying
                | Self::WaitingConfirm
        )
    }

    /// Check if pairing can proceed from this state
    pub fn can_proceed(self) -> bool {
        self.is_active()
    }

    /// Get the next state after user acceptance (for responder)
    pub fn on_accept(self) -> Option<Self> {
        match self {
            Self::IncomingRequest => Some(Self::PendingResponse),
            _ => None,
        }
    }

    /// Get the next state after user rejection
    pub fn on_reject(self) -> Self {
        Self::Rejected
    }

    /// Get the next state after receiving challenge (for initiator)
    pub fn on_challenge(self) -> Option<Self> {
        match self {
            Self::Requesting => Some(Self::PendingChallenge),
            _ => None,
        }
    }

    /// Get the next state after PIN verification
    pub fn on_verify(self, success: bool) -> Self {
        match self {
            Self::PendingChallenge if success => Self::Verifying,
            Self::PendingChallenge => Self::Failed,
            _ => self,
        }
    }

    /// Get the next state after receiving response (for responder)
    pub fn on_response(self, success: bool) -> Self {
        match self {
            Self::PendingResponse if success => Self::WaitingConfirm,
            Self::PendingResponse => Self::Failed,
            _ => self,
        }
    }

    /// Get the next state after confirmation
    pub fn on_confirm(self, success: bool) -> Self {
        match self {
            Self::Verifying | Self::WaitingConfirm if success => Self::Paired,
            Self::Verifying | Self::WaitingConfirm => Self::Failed,
            _ => self,
        }
    }

    /// Mark state as expired
    pub fn expire(self) -> Self {
        if self.is_active() {
            Self::Expired
        } else {
            self
        }
    }
}

impl Default for PairingState {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // State Classification Tests
    // =========================================================================

    #[test]
    fn test_terminal_states() {
        // Terminal states: Paired, Failed, Expired, Rejected
        assert!(PairingState::Paired.is_terminal());
        assert!(PairingState::Failed.is_terminal());
        assert!(PairingState::Expired.is_terminal());
        assert!(PairingState::Rejected.is_terminal());

        // Non-terminal states
        assert!(!PairingState::Idle.is_terminal());
        assert!(!PairingState::Requesting.is_terminal());
        assert!(!PairingState::IncomingRequest.is_terminal());
        assert!(!PairingState::PendingResponse.is_terminal());
        assert!(!PairingState::PendingChallenge.is_terminal());
        assert!(!PairingState::Verifying.is_terminal());
        assert!(!PairingState::WaitingConfirm.is_terminal());
    }

    #[test]
    fn test_active_states() {
        // Active states: Requesting, IncomingRequest, PendingResponse,
        //                PendingChallenge, Verifying, WaitingConfirm
        assert!(PairingState::Requesting.is_active());
        assert!(PairingState::IncomingRequest.is_active());
        assert!(PairingState::PendingResponse.is_active());
        assert!(PairingState::PendingChallenge.is_active());
        assert!(PairingState::Verifying.is_active());
        assert!(PairingState::WaitingConfirm.is_active());

        // Non-active states
        assert!(!PairingState::Idle.is_active());
        assert!(!PairingState::Paired.is_active());
        assert!(!PairingState::Failed.is_active());
        assert!(!PairingState::Expired.is_active());
        assert!(!PairingState::Rejected.is_active());
    }

    #[test]
    fn test_can_proceed() {
        // Active states should allow proceeding
        assert!(PairingState::Requesting.can_proceed());
        assert!(PairingState::PendingChallenge.can_proceed());

        // Terminal and idle states should not allow proceeding
        assert!(!PairingState::Idle.can_proceed());
        assert!(!PairingState::Paired.can_proceed());
        assert!(!PairingState::Failed.can_proceed());
    }

    // =========================================================================
    // Transition Method Tests (on_* helpers)
    // =========================================================================

    #[test]
    fn test_on_accept_responder_flow() {
        // Only IncomingRequest can transition to PendingResponse
        let state = PairingState::IncomingRequest;
        let next = state.on_accept();
        assert_eq!(next, Some(PairingState::PendingResponse));

        // Other states cannot accept
        assert!(PairingState::Idle.on_accept().is_none());
        assert!(PairingState::Requesting.on_accept().is_none());
        assert!(PairingState::Paired.on_accept().is_none());
    }

    #[test]
    fn test_on_reject_any_state() {
        // User rejection always leads to Rejected
        assert_eq!(PairingState::Idle.on_reject(), PairingState::Rejected);
        assert_eq!(
            PairingState::IncomingRequest.on_reject(),
            PairingState::Rejected
        );
        assert_eq!(
            PairingState::PendingChallenge.on_reject(),
            PairingState::Rejected
        );
    }

    #[test]
    fn test_on_challenge_initiator_flow() {
        // Only Requesting can receive challenge
        let state = PairingState::Requesting;
        let next = state.on_challenge();
        assert_eq!(next, Some(PairingState::PendingChallenge));

        // Other states cannot receive challenge
        assert!(PairingState::Idle.on_challenge().is_none());
        assert!(PairingState::IncomingRequest.on_challenge().is_none());
    }

    #[test]
    fn test_on_verify_pin_verification() {
        // Successful verification from PendingChallenge
        let state = PairingState::PendingChallenge;
        assert_eq!(state.on_verify(true), PairingState::Verifying);
        assert_eq!(state.on_verify(false), PairingState::Failed);

        // Wrong state: no transition
        assert_eq!(PairingState::Idle.on_verify(true), PairingState::Idle);
        assert_eq!(
            PairingState::IncomingRequest.on_verify(false),
            PairingState::IncomingRequest
        );
    }

    #[test]
    fn test_on_response_responder_flow() {
        // Response handling in PendingResponse
        let state = PairingState::PendingResponse;
        assert_eq!(state.on_response(true), PairingState::WaitingConfirm);
        assert_eq!(state.on_response(false), PairingState::Failed);

        // Wrong state: no transition
        assert_eq!(PairingState::Idle.on_response(true), PairingState::Idle);
    }

    #[test]
    fn test_on_confirm_final_step() {
        // Both Verifying and WaitingConfirm can confirm
        assert_eq!(
            PairingState::Verifying.on_confirm(true),
            PairingState::Paired
        );
        assert_eq!(
            PairingState::Verifying.on_confirm(false),
            PairingState::Failed
        );
        assert_eq!(
            PairingState::WaitingConfirm.on_confirm(true),
            PairingState::Paired
        );
        assert_eq!(
            PairingState::WaitingConfirm.on_confirm(false),
            PairingState::Failed
        );

        // Wrong states: no transition
        assert_eq!(PairingState::Idle.on_confirm(true), PairingState::Idle);
        assert_eq!(
            PairingState::Requesting.on_confirm(true),
            PairingState::Requesting
        );
    }

    #[test]
    fn test_expire_active_states() {
        // Active states should expire
        assert_eq!(PairingState::Requesting.expire(), PairingState::Expired);
        assert_eq!(
            PairingState::PendingChallenge.expire(),
            PairingState::Expired
        );
        assert_eq!(PairingState::WaitingConfirm.expire(), PairingState::Expired);

        // Non-active states should remain unchanged
        assert_eq!(PairingState::Idle.expire(), PairingState::Idle);
        assert_eq!(PairingState::Paired.expire(), PairingState::Paired);
        assert_eq!(PairingState::Failed.expire(), PairingState::Failed);
    }

    // =========================================================================
    // Event-Based Transition Tests (transition() method)
    // =========================================================================
}
