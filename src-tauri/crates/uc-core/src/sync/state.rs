use serde::{Deserialize, Serialize};

/// Clipboard synchronization state machine
///
/// Design principle: This is a pure type state machine with only state
/// definitions and transition validation logic. Runtime behaviors like
/// retries and timeouts are handled by the application layer (uc-app).
///
/// State transitions:
///
/// ```text
/// Idle
///  │
///  ├─→ Sending ──→ Completed
///  │             └─→ Failed
///  │
///  └─→ Receiving ──→ Processing ──→ Completed
///                               └─→ Conflict ──→ Resolving ──→ Completed
///                                                           └─→ Failed
///
/// All states ──→ Failed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// No sync operation in progress
    Idle,

    /// Sending clipboard content to peers
    Sending,

    /// Receiving clipboard content from a peer
    Receiving,

    /// Processing received content (decryption, validation)
    Processing,

    /// Conflict detected (same content from multiple sources)
    Conflict,

    /// Resolving conflict (user intervention or automatic resolution)
    Resolving,

    /// Sync operation completed successfully
    Completed,

    /// Sync operation failed
    Failed,
}

impl SyncState {
    /// Check if this is a terminal state
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Check if sync is currently in progress
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Sending
                | Self::Receiving
                | Self::Processing
                | Self::Conflict
                | Self::Resolving
        )
    }

    /// Check if we're sending data
    pub fn is_sending(self) -> bool {
        self == Self::Sending
    }

    /// Check if we're receiving data
    pub fn is_receiving(self) -> bool {
        matches!(self, Self::Receiving | Self::Processing)
    }

    /// Start sending operation
    pub fn start_sending(self) -> Option<Self> {
        match self {
            Self::Idle => Some(Self::Sending),
            _ => None,
        }
    }

    /// Start receiving operation
    pub fn start_receiving(self) -> Option<Self> {
        match self {
            Self::Idle => Some(Self::Receiving),
            _ => None,
        }
    }

    /// Transition after sending completes
    pub fn on_sent(self, success: bool) -> Self {
        match self {
            Self::Sending if success => Self::Completed,
            Self::Sending => Self::Failed,
            _ => self,
        }
    }

    /// Transition after receiving starts processing
    pub fn on_received(self) -> Option<Self> {
        match self {
            Self::Receiving => Some(Self::Processing),
            _ => None,
        }
    }

    /// Transition after processing completes
    pub fn on_processed(self, success: bool) -> Self {
        match self {
            Self::Processing if success => Self::Completed,
            Self::Processing => Self::Failed,
            _ => self,
        }
    }

    /// Detect conflict
    pub fn on_conflict(self) -> Option<Self> {
        match self {
            Self::Processing => Some(Self::Conflict),
            _ => None,
        }
    }

    /// Start resolving conflict
    pub fn start_resolving(self) -> Option<Self> {
        match self {
            Self::Conflict => Some(Self::Resolving),
            _ => None,
        }
    }

    /// Transition after conflict resolution
    pub fn on_resolved(self, success: bool) -> Self {
        match self {
            Self::Resolving if success => Self::Completed,
            Self::Resolving => Self::Failed,
            _ => self,
        }
    }

    /// Mark as failed
    pub fn fail(self) -> Self {
        if self.is_active() {
            Self::Failed
        } else {
            self
        }
    }

    /// Reset to idle
    pub fn reset(self) -> Self {
        if self.is_terminal() {
            Self::Idle
        } else {
            self
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_flow() {
        let mut state = SyncState::Idle;

        state = state.start_sending().unwrap();
        assert_eq!(state, SyncState::Sending);
        assert!(state.is_sending());

        state = state.on_sent(true);
        assert_eq!(state, SyncState::Completed);
        assert!(state.is_terminal());
    }

    #[test]
    fn test_receive_flow() {
        let mut state = SyncState::Idle;

        state = state.start_receiving().unwrap();
        assert_eq!(state, SyncState::Receiving);
        assert!(state.is_receiving());

        state = state.on_received().unwrap();
        assert_eq!(state, SyncState::Processing);

        state = state.on_processed(true);
        assert_eq!(state, SyncState::Completed);
    }

    #[test]
    fn test_conflict_resolution() {
        let mut state = SyncState::Processing;

        state = state.on_conflict().unwrap();
        assert_eq!(state, SyncState::Conflict);

        state = state.start_resolving().unwrap();
        assert_eq!(state, SyncState::Resolving);

        state = state.on_resolved(true);
        assert_eq!(state, SyncState::Completed);
    }

    #[test]
    fn test_failed_sync() {
        let state = SyncState::Sending;
        let failed = state.on_sent(false);

        assert_eq!(failed, SyncState::Failed);
        assert!(failed.is_terminal());
    }

    #[test]
    fn test_invalid_transitions() {
        // Can't start sending when not idle
        let state = SyncState::Sending;
        assert!(state.start_sending().is_none());

        // Can't start receiving when not idle
        assert!(state.start_receiving().is_none());
    }

    #[test]
    fn test_reset_from_terminal() {
        let state = SyncState::Completed;
        let reset = state.reset();

        assert_eq!(reset, SyncState::Idle);
    }

    #[test]
    fn test_no_reset_from_active() {
        let state = SyncState::Sending;
        let reset = state.reset();

        assert_eq!(reset, SyncState::Sending);
    }

    #[test]
    fn test_fail_from_active() {
        let state = SyncState::Sending;
        let failed = state.fail();

        assert_eq!(failed, SyncState::Failed);
    }

    #[test]
    fn test_default_state() {
        let state = SyncState::default();
        assert_eq!(state, SyncState::Idle);
    }
}
