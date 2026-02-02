/// Setup error types.
///
/// 设置错误类型。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupError {
    PassphraseMismatch,
    PassphraseEmpty,
    PassphraseInvalidOrMismatch,
    NetworkTimeout,
    PeerUnavailable,
    PairingRejected,
    PairingFailed,
}
