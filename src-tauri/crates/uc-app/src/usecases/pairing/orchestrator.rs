//! Pairing protocol orchestrator
//!
//! 这个模块负责编排配对状态机,将网络事件、用户输入和定时器事件转换为状态机事件,
//! 并执行状态机返回的动作。
//!
//! # Architecture / 架构
//!
//! ```text
//! Network/User/Timer Events
//!   ↓
//! PairingOrchestrator (converts events)
//!   ↓
//! PairingStateMachine (pure state transitions)
//!   ↓
//! PairingActions (executed by orchestrator)
//!   ↓
//! Network/User/Persistence side effects
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

use uc_core::{
    network::{
        pairing_state_machine::{
            PairingAction, PairingEvent, PairingPolicy, PairingRole, PairingStateMachine,
            SessionId, TimeoutKind,
        },
        protocol::{PairingChallenge, PairingConfirm, PairingMessage, PairingRequest},
    },
    ports::PairedDeviceRepositoryPort,
    settings::model::Settings,
};

/// 配对编排器配置
#[derive(Debug, Clone)]
pub struct PairingConfig {
    /// 步骤超时时间(秒)
    pub step_timeout_secs: i64,
    /// 用户确认超时时间(秒)
    pub user_verification_timeout_secs: i64,
    /// 会话超时时间(秒)
    pub session_timeout_secs: i64,
    /// 最大重试次数
    pub max_retries: u8,
    /// 协议版本
    pub protocol_version: String,
}

impl Default for PairingConfig {
    fn default() -> Self {
        Self::from_settings(&Settings::default())
    }
}

impl PairingConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let pairing = &settings.pairing;
        let step = pairing.step_timeout.as_secs().min(i64::MAX as u64) as i64;
        let verify = pairing
            .user_verification_timeout
            .as_secs()
            .min(i64::MAX as u64) as i64;
        let session = pairing.session_timeout.as_secs().min(i64::MAX as u64) as i64;

        Self {
            step_timeout_secs: step.max(1),
            user_verification_timeout_secs: verify.max(1),
            session_timeout_secs: session.max(1),
            max_retries: pairing.max_retries.max(1),
            protocol_version: pairing.protocol_version.clone(),
        }
    }
}

/// 配对编排器
///
/// 负责管理多个并发的配对会话,协调状态机与外部世界的交互。
#[derive(Clone)]
pub struct PairingOrchestrator {
    /// 配置
    config: PairingConfig,
    /// 活跃的配对会话 (session_id -> state machine)
    sessions: Arc<RwLock<HashMap<SessionId, PairingSessionContext>>>,
    /// 会话对应的对端信息
    session_peers: Arc<RwLock<HashMap<SessionId, PairingPeerInfo>>>,
    /// 配对设备仓库
    device_repo: Arc<dyn PairedDeviceRepositoryPort + Send + Sync + 'static>,
    /// 本地设备身份
    local_identity: LocalDeviceInfo,
    /// 动作发送器
    action_tx: mpsc::Sender<PairingAction>,
}

/// 配对会话上下文
struct PairingSessionContext {
    /// 状态机
    state_machine: PairingStateMachine,
    /// 会话创建时间
    created_at: DateTime<Utc>,
    /// 定时器句柄
    timers: Mutex<HashMap<TimeoutKind, tokio::task::AbortHandle>>,
}

#[derive(Debug, Clone)]
pub struct PairingPeerInfo {
    pub peer_id: String,
    pub device_name: Option<String>,
}

/// 本地设备信息
#[derive(Clone)]
struct LocalDeviceInfo {
    /// 设备名称
    device_name: String,
    /// 设备ID
    device_id: String,
    /// 本地PeerID
    peer_id: String,
    /// 本地身份公钥
    identity_pubkey: Vec<u8>,
}

impl PairingOrchestrator {
    /// 创建新的配对编排器
    pub fn new(
        config: PairingConfig,
        device_repo: Arc<dyn PairedDeviceRepositoryPort + Send + Sync + 'static>,
        local_device_name: String,
        local_device_id: String,
        local_peer_id: String,
        local_identity_pubkey: Vec<u8>,
    ) -> (Self, mpsc::Receiver<PairingAction>) {
        let (action_tx, action_rx) = mpsc::channel(100);

        let orchestrator = Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_peers: Arc::new(RwLock::new(HashMap::new())),
            device_repo,
            local_identity: LocalDeviceInfo {
                device_name: local_device_name,
                device_id: local_device_id,
                peer_id: local_peer_id,
                identity_pubkey: local_identity_pubkey,
            },
            action_tx,
        };

        (orchestrator, action_rx)
    }

    /// 发起配对 (Initiator)
    pub async fn initiate_pairing(&self, peer_id: String) -> Result<SessionId> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.record_session_peer(&session_id, peer_id.clone(), None)
            .await;

        let policy = self.build_policy();
        let mut state_machine = PairingStateMachine::new_with_local_identity_and_policy(
            self.local_identity.device_name.clone(),
            self.local_identity.device_id.clone(),
            self.local_identity.identity_pubkey.clone(),
            policy,
        );
        let _ = state_machine.handle_event(
            PairingEvent::StartPairing {
                role: PairingRole::Initiator,
                peer_id: peer_id.clone(),
            },
            Utc::now(),
        );

        let context = PairingSessionContext {
            state_machine,
            created_at: Utc::now(),
            timers: Mutex::new(HashMap::new()),
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), context);

        let nonce = uuid::Uuid::new_v4().as_bytes().to_vec();
        let request = PairingRequest {
            session_id: session_id.clone(),
            device_name: self.local_identity.device_name.clone(),
            device_id: self.local_identity.device_id.clone(),
            peer_id: self.local_identity.peer_id.clone(),
            identity_pubkey: self.local_identity.identity_pubkey.clone(),
            nonce,
        };

        self.send_message(&peer_id, PairingMessage::Request(request))
            .await?;

        Ok(session_id)
    }

    /// 处理收到的配对请求 (Responder)
    pub async fn handle_incoming_request(
        &self,
        peer_id: String,
        request: PairingRequest,
    ) -> Result<()> {
        let session_id = request.session_id.clone();
        self.record_session_peer(
            &session_id,
            peer_id.clone(),
            Some(request.device_name.clone()),
        )
        .await;

        let policy = self.build_policy();
        let mut state_machine = PairingStateMachine::new_with_local_identity_and_policy(
            self.local_identity.device_name.clone(),
            self.local_identity.device_id.clone(),
            self.local_identity.identity_pubkey.clone(),
            policy,
        );
        let (_state, actions) = state_machine.handle_event(
            PairingEvent::RecvRequest {
                session_id: session_id.clone(),
                request,
            },
            Utc::now(),
        );

        // 执行动作(如果有)
        for action in actions {
            self.execute_action(&session_id, &peer_id, action).await?;
        }

        let context = PairingSessionContext {
            state_machine,
            created_at: Utc::now(),
            timers: Mutex::new(HashMap::new()),
        };

        self.sessions.write().await.insert(session_id, context);

        Ok(())
    }

    /// 处理收到的Challenge (Initiator)
    pub async fn handle_challenge(
        &self,
        session_id: &str,
        peer_id: &str,
        challenge: PairingChallenge,
    ) -> Result<()> {
        self.record_session_peer(
            session_id,
            peer_id.to_string(),
            Some(challenge.device_name.clone()),
        )
        .await;
        let actions = {
            let mut sessions = self.sessions.write().await;
            let context = sessions.get_mut(session_id).context("Session not found")?;
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::RecvChallenge {
                    session_id: session_id.to_string(),
                    challenge,
                },
                Utc::now(),
            );
            actions
        };

        // 执行动作(包括展示验证UI)
        for action in actions {
            self.execute_action(session_id, peer_id, action).await?;
        }

        Ok(())
    }

    /// 处理收到的Response (Responder)
    pub async fn handle_response(
        &self,
        session_id: &str,
        peer_id: &str,
        response: uc_core::network::protocol::PairingResponse,
    ) -> Result<()> {
        let actions = {
            let mut sessions = self.sessions.write().await;
            let context = sessions.get_mut(session_id).context("Session not found")?;
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::RecvResponse {
                    session_id: session_id.to_string(),
                    response,
                },
                Utc::now(),
            );
            actions
        };

        for action in actions {
            self.execute_action(session_id, peer_id, action).await?;
        }

        Ok(())
    }

    /// 用户接受配对 (验证短码匹配)
    pub async fn user_accept_pairing(&self, session_id: &str) -> Result<()> {
        let actions = {
            let mut sessions = self.sessions.write().await;
            let context = sessions.get_mut(session_id).context("Session not found")?;
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::UserAccept {
                    session_id: session_id.to_string(),
                },
                Utc::now(),
            );
            actions
        };

        for action in actions {
            self.execute_action(session_id, "", action).await?;
        }

        Ok(())
    }

    /// 用户拒绝配对
    pub async fn user_reject_pairing(&self, session_id: &str) -> Result<()> {
        let actions = {
            let mut sessions = self.sessions.write().await;
            let context = sessions.get_mut(session_id).context("Session not found")?;
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::UserReject {
                    session_id: session_id.to_string(),
                },
                Utc::now(),
            );
            actions
        };

        for action in actions {
            self.execute_action(session_id, "", action).await?;
        }

        Ok(())
    }

    /// 处理收到的Confirm (双方)
    pub async fn handle_confirm(
        &self,
        session_id: &str,
        peer_id: &str,
        confirm: PairingConfirm,
    ) -> Result<()> {
        let actions = {
            let mut sessions = self.sessions.write().await;
            let context = sessions.get_mut(session_id).context("Session not found")?;
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::RecvConfirm {
                    session_id: session_id.to_string(),
                    confirm,
                },
                Utc::now(),
            );
            actions
        };

        for action in actions {
            self.execute_action(session_id, peer_id, action).await?;
        }

        Ok(())
    }

    /// 执行单个动作
    async fn execute_action(
        &self,
        session_id: &str,
        _peer_id: &str,
        action: PairingAction,
    ) -> Result<()> {
        Self::execute_action_inner(
            self.action_tx.clone(),
            self.sessions.clone(),
            self.device_repo.clone(),
            session_id.to_string(),
            action,
        )
        .await
    }

    fn execute_action_inner(
        action_tx: mpsc::Sender<PairingAction>,
        sessions: Arc<RwLock<HashMap<SessionId, PairingSessionContext>>>,
        device_repo: Arc<dyn PairedDeviceRepositoryPort + Send + Sync + 'static>,
        session_id: String,
        action: PairingAction,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            match action {
                PairingAction::Send {
                    peer_id: target_peer,
                    message,
                } => {
                    action_tx
                        .send(PairingAction::Send {
                            peer_id: target_peer,
                            message,
                        })
                        .await
                        .context("Failed to queue send action")?;
                }
                PairingAction::ShowVerification { .. } | PairingAction::EmitResult { .. } => {
                    action_tx
                        .send(action)
                        .await
                        .context("Failed to queue ui action")?;
                }
                PairingAction::PersistPairedDevice {
                    session_id: _,
                    device,
                } => {
                    device_repo
                        .upsert(device.clone())
                        .await
                        .context("Failed to persist paired device")?;

                    // 通知状态机持久化成功
                    let mut sessions = sessions.write().await;
                    if let Some(context) = sessions.get_mut(&session_id) {
                        let _ = context.state_machine.handle_event(
                            PairingEvent::PersistOk {
                                session_id: session_id.clone(),
                                device_id: device.peer_id.to_string(),
                            },
                            Utc::now(),
                        );
                    }
                }
                PairingAction::StartTimer {
                    session_id: action_session_id,
                    kind,
                    deadline,
                } => {
                    let sessions_for_timer = sessions.clone();
                    let mut sessions = sessions.write().await;
                    let context = sessions
                        .get_mut(&action_session_id)
                        .context("Session not found")?;
                    {
                        let mut timers = context.timers.lock().await;
                        if let Some(handle) = timers.remove(&kind) {
                            handle.abort();
                        }
                    }

                    let action_tx = action_tx.clone();
                    let sessions = sessions_for_timer;
                    let device_repo = device_repo.clone();
                    let session_id_for_log = action_session_id.clone();
                    let sleep_duration = deadline
                        .signed_duration_since(Utc::now())
                        .to_std()
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
                    let future = async move {
                        tokio::time::sleep(sleep_duration).await;
                        if let Err(error) = Self::handle_timeout(
                            action_tx,
                            sessions,
                            device_repo,
                            action_session_id,
                            kind,
                        )
                        .await
                        {
                            tracing::error!(
                                %session_id_for_log,
                                ?kind,
                                error = ?error,
                                "pairing timer handling failed"
                            );
                        }
                    };
                    let future: Pin<Box<dyn Future<Output = ()> + Send>> = Box::pin(future);
                    let handle = tokio::spawn(future);

                    let abort_handle = handle.abort_handle();
                    let mut timers = context.timers.lock().await;
                    timers.insert(kind, abort_handle);
                }
                PairingAction::CancelTimer {
                    session_id: action_session_id,
                    kind,
                } => {
                    let mut sessions = sessions.write().await;
                    let context = sessions
                        .get_mut(&action_session_id)
                        .context("Session not found")?;
                    let mut timers = context.timers.lock().await;
                    if let Some(handle) = timers.remove(&kind) {
                        handle.abort();
                    }
                }
                PairingAction::LogTransition { .. } => {
                    // 日志已记录,无需额外操作
                }
                PairingAction::NoOp => {}
            }

            Ok(())
        }
    }

    async fn record_session_peer(
        &self,
        session_id: &str,
        peer_id: String,
        device_name: Option<String>,
    ) {
        let mut peers = self.session_peers.write().await;
        let entry = peers
            .entry(session_id.to_string())
            .or_insert_with(|| PairingPeerInfo {
                peer_id: peer_id.clone(),
                device_name: None,
            });
        entry.peer_id = peer_id;
        if device_name.is_some() {
            entry.device_name = device_name;
        }
    }

    pub async fn get_session_peer(&self, session_id: &str) -> Option<PairingPeerInfo> {
        let peers = self.session_peers.read().await;
        peers.get(session_id).cloned()
    }

    /// 发送消息到对端
    async fn send_message(&self, peer_id: &str, message: PairingMessage) -> Result<()> {
        // TODO: 通过网络层发送
        let action_tx = self.action_tx.clone();
        action_tx
            .send(PairingAction::Send {
                peer_id: peer_id.to_string(),
                message,
            })
            .await
            .context("Failed to queue send action")?;
        Ok(())
    }

    fn build_policy(&self) -> PairingPolicy {
        PairingPolicy {
            step_timeout_secs: self.config.step_timeout_secs,
            user_verification_timeout_secs: self.config.user_verification_timeout_secs,
            max_retries: self.config.max_retries,
            protocol_version: self.config.protocol_version.clone(),
        }
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let timeout = Duration::seconds(self.config.session_timeout_secs);

        sessions.retain(|_, context| {
            now.signed_duration_since(context.created_at).num_seconds() < timeout.num_seconds()
        });
    }

    async fn handle_timeout(
        action_tx: mpsc::Sender<PairingAction>,
        sessions: Arc<RwLock<HashMap<SessionId, PairingSessionContext>>>,
        device_repo: Arc<dyn PairedDeviceRepositoryPort + Send + Sync + 'static>,
        session_id: String,
        kind: TimeoutKind,
    ) -> Result<()> {
        let actions = {
            let mut sessions = sessions.write().await;
            let context = sessions.get_mut(&session_id).context("Session not found")?;
            {
                let mut timers = context.timers.lock().await;
                timers.remove(&kind);
            }
            let (_state, actions) = context.state_machine.handle_event(
                PairingEvent::Timeout {
                    session_id: session_id.clone(),
                    kind,
                },
                Utc::now(),
            );
            actions
        };

        for action in actions {
            Self::execute_action_inner(
                action_tx.clone(),
                sessions.clone(),
                device_repo.clone(),
                session_id.clone(),
                action,
            )
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;
    use uc_core::crypto::pin_hash::hash_pin;
    use uc_core::network::paired_device::{PairedDevice, PairingState};
    use uc_core::network::protocol::{PairingRequest, PairingResponse};

    struct MockDeviceRepository;

    #[async_trait::async_trait]
    impl PairedDeviceRepositoryPort for MockDeviceRepository {
        async fn get_by_peer_id(
            &self,
            _peer_id: &uc_core::ids::PeerId,
        ) -> Result<Option<PairedDevice>, uc_core::ports::errors::PairedDeviceRepositoryError>
        {
            Ok(None)
        }

        async fn list_all(
            &self,
        ) -> Result<Vec<PairedDevice>, uc_core::ports::errors::PairedDeviceRepositoryError>
        {
            Ok(vec![])
        }

        async fn upsert(
            &self,
            _device: PairedDevice,
        ) -> Result<(), uc_core::ports::errors::PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn set_state(
            &self,
            _peer_id: &uc_core::ids::PeerId,
            _state: PairingState,
        ) -> Result<(), uc_core::ports::errors::PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn update_last_seen(
            &self,
            _peer_id: &uc_core::ids::PeerId,
            _last_seen_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<(), uc_core::ports::errors::PairedDeviceRepositoryError> {
            Ok(())
        }

        async fn delete(
            &self,
            _peer_id: &uc_core::ids::PeerId,
        ) -> Result<(), uc_core::ports::errors::PairedDeviceRepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (_orchestrator, _action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "TestDevice".to_string(),
            "device-123".to_string(),
            "peer-456".to_string(),
            vec![0u8; 32],
        );
    }

    #[tokio::test]
    async fn test_initiate_pairing() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, _action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "TestDevice".to_string(),
            "device-123".to_string(),
            "peer-456".to_string(),
            vec![0u8; 32],
        );

        let session_id = orchestrator
            .initiate_pairing("remote-peer".to_string())
            .await
            .unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_cleanup_uses_configured_session_timeout() {
        let config = PairingConfig {
            step_timeout_secs: 1,
            user_verification_timeout_secs: 1,
            session_timeout_secs: 1,
            max_retries: 1,
            protocol_version: "1.0.0".to_string(),
        };
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, _action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "TestDevice".to_string(),
            "device-123".to_string(),
            "peer-456".to_string(),
            vec![0u8; 32],
        );

        orchestrator
            .initiate_pairing("remote-peer".to_string())
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        orchestrator.cleanup_expired_sessions().await;

        let sessions = orchestrator.sessions.read().await;
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_pairing_config_from_settings() {
        let mut settings = Settings::default();
        settings.pairing.step_timeout = std::time::Duration::from_secs(20);
        settings.pairing.user_verification_timeout = std::time::Duration::from_secs(90);
        settings.pairing.session_timeout = std::time::Duration::from_secs(400);
        settings.pairing.max_retries = 5;
        settings.pairing.protocol_version = "2.0.0".to_string();

        let config = PairingConfig::from_settings(&settings);

        assert_eq!(config.step_timeout_secs, 20);
        assert_eq!(config.user_verification_timeout_secs, 90);
        assert_eq!(config.session_timeout_secs, 400);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.protocol_version, "2.0.0");
    }

    #[tokio::test]
    async fn test_handle_response_emits_confirm_action() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, mut action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "LocalDevice".to_string(),
            "device-123".to_string(),
            "peer-local".to_string(),
            vec![0u8; 32],
        );

        let request = PairingRequest {
            session_id: "session-1".to_string(),
            device_name: "PeerDevice".to_string(),
            device_id: "device-999".to_string(),
            peer_id: "peer-remote".to_string(),
            identity_pubkey: vec![1; 32],
            nonce: vec![2; 16],
        };

        orchestrator
            .handle_incoming_request("peer-remote".to_string(), request)
            .await
            .expect("handle request");
        orchestrator
            .user_accept_pairing("session-1")
            .await
            .expect("accept pairing");

        let challenge_action = timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("challenge action timeout")
            .expect("challenge action missing");
        let challenge = match challenge_action {
            PairingAction::Send {
                message: PairingMessage::Challenge(challenge),
                ..
            } => challenge,
            other => panic!("unexpected action: {other:?}"),
        };

        let pin_hash = hash_pin(&challenge.pin).expect("hash pin");
        let response = PairingResponse {
            session_id: challenge.session_id.clone(),
            pin_hash,
            accepted: true,
        };

        orchestrator
            .handle_response(&challenge.session_id, "peer-remote", response)
            .await
            .expect("handle response");

        let confirm_action = timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("confirm action timeout")
            .expect("confirm action missing");
        let confirm = match confirm_action {
            PairingAction::Send {
                message: PairingMessage::Confirm(confirm),
                ..
            } => confirm,
            other => panic!("unexpected action: {other:?}"),
        };

        assert!(confirm.success);
        assert_eq!(confirm.sender_device_name, "LocalDevice");
        assert_eq!(confirm.device_id, "device-123");
    }

    #[tokio::test]
    async fn test_show_verification_is_forwarded_to_action_channel() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, mut action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "Local".to_string(),
            "device-1".to_string(),
            "peer-local".to_string(),
            vec![1; 32],
        );

        orchestrator
            .execute_action(
                "session-1",
                "peer-remote",
                PairingAction::ShowVerification {
                    session_id: "session-1".to_string(),
                    short_code: "ABC123".to_string(),
                    local_fingerprint: "LOCAL".to_string(),
                    peer_fingerprint: "PEER".to_string(),
                    peer_display_name: "Peer".to_string(),
                },
            )
            .await
            .expect("execute action");

        let action = timeout(Duration::from_secs(1), action_rx.recv())
            .await
            .expect("action timeout")
            .expect("action missing");

        assert!(matches!(action, PairingAction::ShowVerification { .. }));
    }

    #[tokio::test]
    async fn test_start_timer_records_handle() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, _action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "Local".to_string(),
            "device-1".to_string(),
            "peer-1".to_string(),
            vec![1; 32],
        );

        let session_id = orchestrator
            .initiate_pairing("peer-2".to_string())
            .await
            .unwrap();
        orchestrator
            .execute_action(
                &session_id,
                "peer-2",
                PairingAction::StartTimer {
                    session_id: session_id.clone(),
                    kind: TimeoutKind::WaitingChallenge,
                    deadline: Utc::now() + chrono::Duration::seconds(1),
                },
            )
            .await
            .unwrap();

        let sessions = orchestrator.sessions.read().await;
        let context = sessions.get(&session_id).expect("session");
        let timers = context.timers.lock().await;
        assert!(timers.contains_key(&TimeoutKind::WaitingChallenge));
    }

    #[tokio::test]
    async fn test_cancel_timer_removes_handle() {
        let config = PairingConfig::default();
        let device_repo = Arc::new(MockDeviceRepository);
        let (orchestrator, _action_rx) = PairingOrchestrator::new(
            config,
            device_repo,
            "Local".to_string(),
            "device-1".to_string(),
            "peer-1".to_string(),
            vec![1; 32],
        );

        let session_id = orchestrator
            .initiate_pairing("peer-2".to_string())
            .await
            .unwrap();
        orchestrator
            .execute_action(
                &session_id,
                "peer-2",
                PairingAction::StartTimer {
                    session_id: session_id.clone(),
                    kind: TimeoutKind::WaitingChallenge,
                    deadline: Utc::now() + chrono::Duration::seconds(1),
                },
            )
            .await
            .unwrap();

        {
            let sessions = orchestrator.sessions.read().await;
            let context = sessions.get(&session_id).expect("session");
            let timers = context.timers.lock().await;
            assert!(timers.contains_key(&TimeoutKind::WaitingChallenge));
        }
        orchestrator
            .execute_action(
                &session_id,
                "peer-2",
                PairingAction::CancelTimer {
                    session_id: session_id.clone(),
                    kind: TimeoutKind::WaitingChallenge,
                },
            )
            .await
            .unwrap();

        let sessions = orchestrator.sessions.read().await;
        let context = sessions.get(&session_id).expect("session");
        let timers = context.timers.lock().await;
        assert!(!timers.contains_key(&TimeoutKind::WaitingChallenge));
    }
}
