//! Setup state machine.
//!
//! Defines a pure state transition function for the onboarding setup flow.

/// Setup flow state.
///
/// 设置流程状态。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupState {
    /// Welcome screen.
    ///
    /// 欢迎页。
    Welcome,
    /// Create-space passphrase input.
    ///
    /// 创建空间口令输入页。
    CreateSpacePassphrase { error: Option<SetupError> },
    /// Join-space device picker.
    ///
    /// 加入空间设备选择页。
    JoinSpacePickDevice { error: Option<SetupError> },
    /// Join-space passphrase verification.
    ///
    /// 加入空间口令验证页。
    JoinSpaceVerifyPassphrase {
        peer_id: String,
        error: Option<SetupError>,
    },
    /// Pairing confirmation (short code).
    ///
    /// 配对确认页（短码）。
    PairingConfirm {
        session_id: String,
        short_code: String,
        peer_fingerprint: Option<String>,
        error: Option<SetupError>,
    },
    /// Setup completed.
    ///
    /// 设置完成。
    Done,
}

/// Events that drive the setup flow.
///
/// 驱动设置流程的事件。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupEvent {
    /// User chooses to create a new space.
    ///
    /// 用户选择创建新空间。
    ChooseCreateSpace,
    /// User chooses to join an existing space.
    ///
    /// 用户选择加入已有空间。
    ChooseJoinSpace,
    /// Navigate back.
    ///
    /// 返回。
    Back,
    /// Submit passphrase for creating a space.
    ///
    /// 提交创建空间口令。
    SubmitCreatePassphrase { pass1: String, pass2: String },
    /// Select a peer device.
    ///
    /// 选择设备。
    SelectPeer { peer_id: String },
    /// Submit passphrase for joining a space.
    ///
    /// 提交加入空间口令。
    SubmitJoinPassphrase { passphrase: String },
    /// User confirms pairing.
    ///
    /// 用户确认配对。
    PairingUserConfirm,
    /// User cancels pairing.
    ///
    /// 用户取消配对。
    PairingUserCancel,
    /// Pairing succeeded (network).
    ///
    /// 配对成功（网络回调）。
    PairingSucceeded,
    /// Pairing failed (network).
    ///
    /// 配对失败（网络回调）。
    PairingFailed { reason: SetupError },
    /// Passphrase mismatch when joining.
    ///
    /// 加入口令不一致。
    PassphraseMismatch,
    /// Refresh network scan.
    ///
    /// 重新扫描设备。
    NetworkScanRefresh,
}

/// Side-effects produced by state transitions.
///
/// 状态迁移产生的副作用。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupAction {
    /// Create a new encrypted space.
    ///
    /// 创建新的加密空间。
    CreateEncryptedSpace { passphrase: String },
    /// Scan peers via network.
    ///
    /// 扫描设备。
    ScanPeers,
    /// Verify passphrase with a peer.
    ///
    /// 与设备验证口令。
    VerifyPassphraseWithPeer { peer_id: String, passphrase: String },
    /// Start pairing with peer.
    ///
    /// 启动配对。
    StartPairing { peer_id: String },
    /// Confirm pairing session.
    ///
    /// 确认配对会话。
    ConfirmPairing { session_id: String },
    /// Cancel pairing session.
    ///
    /// 取消配对会话。
    CancelPairing { session_id: String },
    /// Mark setup completed.
    ///
    /// 标记设置完成。
    MarkSetupComplete,
}

/// Setup error types.
///
/// 设置错误类型。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupError {
    PassphraseMismatch,
    PassphraseTooShort { min_len: usize },
    PassphraseEmpty,
    PassphraseInvalidOrMismatch,
    NetworkTimeout,
    PeerUnavailable,
    PairingRejected,
    PairingFailed,
}

/// Pure setup state machine.
///
/// 纯状态机：不包含副作用。
pub struct SetupStateMachine;

impl SetupStateMachine {
    pub fn transition(state: SetupState, event: SetupEvent) -> (SetupState, Vec<SetupAction>) {
        match (state, event) {
            (SetupState::Welcome, SetupEvent::ChooseCreateSpace) => (
                SetupState::CreateSpacePassphrase { error: None },
                Vec::new(),
            ),
            (SetupState::Welcome, SetupEvent::ChooseJoinSpace) => {
                (SetupState::JoinSpacePickDevice { error: None }, Vec::new())
            }
            (SetupState::CreateSpacePassphrase { .. }, SetupEvent::Back) => {
                (SetupState::Welcome, Vec::new())
            }
            (
                SetupState::CreateSpacePassphrase { .. },
                SetupEvent::SubmitCreatePassphrase { pass1, pass2 },
            ) => {
                if pass1.is_empty() {
                    return (
                        SetupState::CreateSpacePassphrase {
                            error: Some(SetupError::PassphraseEmpty),
                        },
                        Vec::new(),
                    );
                }
                if pass1.len() < MIN_PASSPHRASE_LEN {
                    return (
                        SetupState::CreateSpacePassphrase {
                            error: Some(SetupError::PassphraseTooShort {
                                min_len: MIN_PASSPHRASE_LEN,
                            }),
                        },
                        Vec::new(),
                    );
                }
                if pass1 != pass2 {
                    return (
                        SetupState::CreateSpacePassphrase {
                            error: Some(SetupError::PassphraseMismatch),
                        },
                        Vec::new(),
                    );
                }
                (
                    SetupState::Done,
                    vec![
                        SetupAction::CreateEncryptedSpace { passphrase: pass1 },
                        SetupAction::MarkSetupComplete,
                    ],
                )
            }
            (SetupState::JoinSpacePickDevice { .. }, SetupEvent::Back) => {
                (SetupState::Welcome, Vec::new())
            }
            (SetupState::JoinSpacePickDevice { .. }, SetupEvent::NetworkScanRefresh) => (
                SetupState::JoinSpacePickDevice { error: None },
                vec![SetupAction::ScanPeers],
            ),
            (SetupState::JoinSpacePickDevice { .. }, SetupEvent::SelectPeer { peer_id }) => (
                SetupState::JoinSpaceVerifyPassphrase {
                    peer_id,
                    error: None,
                },
                Vec::new(),
            ),
            (
                SetupState::JoinSpaceVerifyPassphrase { peer_id, .. },
                SetupEvent::SubmitJoinPassphrase { passphrase },
            ) => (
                SetupState::JoinSpaceVerifyPassphrase {
                    peer_id: peer_id.clone(),
                    error: None,
                },
                vec![SetupAction::VerifyPassphraseWithPeer {
                    peer_id,
                    passphrase,
                }],
            ),
            (
                SetupState::JoinSpaceVerifyPassphrase { peer_id, .. },
                SetupEvent::PassphraseMismatch,
            ) => (
                SetupState::JoinSpaceVerifyPassphrase {
                    peer_id,
                    error: Some(SetupError::PassphraseInvalidOrMismatch),
                },
                Vec::new(),
            ),
            (SetupState::JoinSpaceVerifyPassphrase { .. }, SetupEvent::Back) => {
                (SetupState::JoinSpacePickDevice { error: None }, Vec::new())
            }
            (
                SetupState::PairingConfirm {
                    session_id,
                    short_code,
                    peer_fingerprint,
                    error,
                },
                SetupEvent::PairingUserConfirm,
            ) => (
                SetupState::PairingConfirm {
                    session_id,
                    short_code,
                    peer_fingerprint,
                    error,
                },
                Vec::new(),
            ),
            (SetupState::PairingConfirm { .. }, SetupEvent::PairingUserCancel) => {
                (SetupState::JoinSpacePickDevice { error: None }, Vec::new())
            }
            (SetupState::PairingConfirm { .. }, SetupEvent::PairingSucceeded) => {
                (SetupState::Done, vec![SetupAction::MarkSetupComplete])
            }
            (SetupState::PairingConfirm { .. }, SetupEvent::PairingFailed { reason }) => (
                SetupState::JoinSpacePickDevice {
                    error: Some(reason),
                },
                Vec::new(),
            ),
            (state, _event) => (state, Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SetupError, SetupEvent, SetupState, SetupStateMachine};

    #[test]
    fn setup_state_machine_welcome_choose_create_transitions_to_create_passphrase() {
        let state = SetupState::Welcome;
        let (next, actions) = SetupStateMachine::transition(state, SetupEvent::ChooseCreateSpace);
        assert_eq!(next, SetupState::CreateSpacePassphrase { error: None });
        assert!(actions.is_empty());
    }

    #[test]
    fn setup_state_machine_create_passphrase_mismatch_sets_error() {
        let state = SetupState::CreateSpacePassphrase { error: None };
        let event = SetupEvent::SubmitCreatePassphrase {
            pass1: "password1".into(),
            pass2: "password2".into(),
        };
        let (next, actions) = SetupStateMachine::transition(state, event);
        assert_eq!(
            next,
            SetupState::CreateSpacePassphrase {
                error: Some(SetupError::PassphraseMismatch)
            }
        );
        assert!(actions.is_empty());
    }
}

const MIN_PASSPHRASE_LEN: usize = 8;
