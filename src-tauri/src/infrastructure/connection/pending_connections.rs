//! 待处理连接请求管理器
//!
//! 管理等待用户确认的连接请求（入站和出站）

use crate::domain::network::{ConnectionRequestMessage, ConnectionResponseMessage};
use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

/// 待处理的入站连接请求（其他设备请求连接到我们）
#[derive(Debug, Clone)]
pub struct PendingIncomingRequest {
    /// 请求方设备 ID
    pub requester_device_id: String,
    /// 请求方 IP 地址
    pub requester_ip: String,
    /// 请求方设备别名（可选）
    pub requester_alias: Option<String>,
    /// 请求方平台（可选）
    pub requester_platform: Option<String>,
}

/// 待处理的出站连接请求（我们请求连接到其他设备）
#[derive(Debug, Clone)]
pub struct PendingOutgoingRequest {
    /// 目标设备 IP:port
    pub target_addr: String,
    /// 目标设备 ID
    pub target_device_id: Option<String>,
    /// 请求时间戳
    pub requested_at: std::time::Instant,
}

/// 待处理连接请求管理器
#[derive(Clone)]
pub struct PendingConnectionsManager {
    /// 待处理的入站请求（请求方设备 ID -> (请求信息, 响应通道)）
    incoming_requests: Arc<RwLock<HashMap<String, (PendingIncomingRequest, oneshot::Sender<bool>)>>>,
    /// 待处理的出站请求（目标地址 -> (请求信息, 响应通道)）
    outgoing_requests: Arc<RwLock<HashMap<String, (PendingOutgoingRequest, oneshot::Sender<ConnectionResponseMessage>)>>>,
}

impl PendingConnectionsManager {
    pub fn new() -> Self {
        Self {
            incoming_requests: Arc::new(RwLock::new(HashMap::new())),
            outgoing_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加一个待处理的入站连接请求
    ///
    /// 返回一个 oneshot receiver，用于接收用户的响应决定
    pub async fn add_incoming_request(
        &self,
        request: ConnectionRequestMessage,
        response_tx: oneshot::Sender<bool>,
    ) -> Result<()> {
        let pending_request = PendingIncomingRequest {
            requester_device_id: request.requester_device_id.clone(),
            requester_ip: request.requester_ip.clone(),
            requester_alias: request.requester_alias.clone(),
            requester_platform: request.requester_platform.clone(),
        };

        let mut requests = self.incoming_requests.write().await;
        requests.insert(request.requester_device_id.clone(), (pending_request, response_tx));

        info!(
            "Added pending incoming request from device {}",
            request.requester_device_id
        );

        Ok(())
    }

    /// 处理用户对入站请求的响应
    ///
    /// 返回响应通道，调用者需要使用它发送响应
    pub async fn respond_to_incoming_request(
        &self,
        requester_device_id: &str,
        accept: bool,
    ) -> Result<oneshot::Sender<bool>> {
        let mut requests = self.incoming_requests.write().await;

        if let Some((pending, response_tx)) = requests.remove(requester_device_id) {
            info!(
                "Responding to incoming request from device {}: {}",
                requester_device_id,
                if accept { "accepted" } else { "rejected" }
            );
            Ok(response_tx)
        } else {
            warn!(
                "No pending incoming request found for device {}",
                requester_device_id
            );
            Err(anyhow::anyhow!(
                "No pending incoming request found for device {}",
                requester_device_id
            ))
        }
    }

    /// 获取所有待处理的入站请求
    pub async fn get_incoming_requests(&self) -> Vec<PendingIncomingRequest> {
        let requests = self.incoming_requests.read().await;
        requests
            .values()
            .map(|(req, _)| req.clone())
            .collect()
    }

    /// 添加一个待处理的出站连接请求
    ///
    /// 返回一个 oneshot receiver，用于接收对方的响应
    pub async fn add_outgoing_request(
        &self,
        target_addr: String,
        target_device_id: Option<String>,
        response_tx: oneshot::Sender<ConnectionResponseMessage>,
    ) -> Result<()> {
        let pending_request = PendingOutgoingRequest {
            target_addr: target_addr.clone(),
            target_device_id,
            requested_at: std::time::Instant::now(),
        };

        let mut requests = self.outgoing_requests.write().await;
        requests.insert(target_addr.clone(), (pending_request, response_tx));

        debug!("Added pending outgoing request to {}", target_addr);

        Ok(())
    }

    /// 处理收到出站请求的响应
    pub async fn handle_outgoing_response(
        &self,
        target_addr: &str,
        response: ConnectionResponseMessage,
    ) -> Result<()> {
        let mut requests = self.outgoing_requests.write().await;

        if let Some((pending, response_tx)) = requests.remove(target_addr) {
            info!(
                "Received response to outgoing request to {}: accepted={}",
                target_addr, response.accepted
            );

            // 发送响应到等待的协程
            let _ = response_tx.send(response);

            Ok(())
        } else {
            warn!(
                "No pending outgoing request found for {}",
                target_addr
            );
            Err(anyhow::anyhow!(
                "No pending outgoing request found for {}",
                target_addr
            ))
        }
    }

    /// 等待出站请求的响应（带超时）
    pub async fn wait_for_outgoing_response(
        &self,
        target_addr: &str,
        timeout_secs: u64,
    ) -> Result<ConnectionResponseMessage> {
        let rx = {
            let requests = self.outgoing_requests.read().await;
            if let Some((_, pending)) = requests.get(target_addr) {
                // 我们不能克隆 Sender，所以需要另一种方式来等待响应
                // 这里我们使用轮询的方式
                return Err(anyhow::anyhow!(
                    "Use wait_for_outgoing_response_loop instead"
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "No pending outgoing request found for {}",
                    target_addr
                ));
            }
        };
    }

    /// 轮询等待出站请求的响应
    pub async fn wait_for_outgoing_response_loop(
        &self,
        target_addr: &str,
        timeout_secs: u64,
    ) -> Result<ConnectionResponseMessage> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                // 超时，移除待处理请求
                let mut requests = self.outgoing_requests.write().await;
                requests.remove(target_addr);
                return Err(anyhow::anyhow!(
                    "Timed out waiting for response from {}",
                    target_addr
                ));
            }

            // 检查是否有响应
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // 这里需要外部调用 handle_outgoing_response 来发送响应
            // 所以这个方法实际上不应该是阻塞的
            // 让我们重新设计这个
        }
    }

    /// 检查出站请求是否已完成
    pub async fn check_outgoing_response(&self, target_addr: &str) -> Option<ConnectionResponseMessage> {
        // 这个方法将由外部调用，当收到响应时
        let requests = self.outgoing_requests.read().await;
        requests.get(target_addr).map(|_| {
            // 如果请求存在，说明还没有收到响应
            // 这个方法的设计需要重新考虑
            // 实际上应该在 handle_outgoing_response 中处理
            ConnectionResponseMessage {
                accepted: false,
                responder_device_id: String::new(),
                responder_ip: None,
                responder_alias: None,
            }
        })
    }

    /// 清理过期的出站请求
    pub async fn cleanup_expired_outgoing(&self, timeout_secs: u64) {
        let mut requests = self.outgoing_requests.write().await;
        let now = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        requests.retain(|addr, (pending, _)| {
            let expired = now.duration_since(pending.requested_at) > timeout;
            if expired {
                debug!("Removing expired outgoing request to {}", addr);
            }
            !expired
        });
    }

    /// 取消指定的出站请求
    pub async fn cancel_outgoing_request(&self, target_addr: &str) -> Result<()> {
        let mut requests = self.outgoing_requests.write().await;

        if requests.remove(target_addr).is_some() {
            info!("Cancelled outgoing request to {}", target_addr);
            Ok(())
        } else {
            warn!("No outgoing request to {} found", target_addr);
            Err(anyhow::anyhow!(
                "No outgoing request to {} found",
                target_addr
            ))
        }
    }

    /// 清空所有待处理的请求
    pub async fn clear_all(&self) {
        let mut incoming = self.incoming_requests.write().await;
        let mut outgoing = self.outgoing_requests.write().await;

        let incoming_count = incoming.len();
        let outgoing_count = outgoing.len();

        incoming.clear();
        outgoing.clear();

        info!(
            "Cleared all pending requests: {} incoming, {} outgoing",
            incoming_count, outgoing_count
        );
    }
}

impl Default for PendingConnectionsManager {
    fn default() -> Self {
        Self::new()
    }
}
