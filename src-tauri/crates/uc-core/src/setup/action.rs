#[derive(Debug, PartialEq)]
pub enum SetupAction {
    /// Ensure device discovery is running (idempotent).
    EnsureDiscovery,

    /// Ensure pairing session with selected peer.
    EnsurePairing,

    /// User confirms the peer identity (short code / fingerprint).
    ConfirmPeerTrust,

    /// Abort current pairing/session.
    AbortPairing,

    /// Create a new encrypted space (uses passphrase from context).
    CreateEncryptedSpace,

    /// Start join-space access protocol (SpaceAccess).
    StartJoinSpaceAccess,

    /// Mark setup flow completed.
    MarkSetupComplete,
}
