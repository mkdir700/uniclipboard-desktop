//! 网络相关的域模型
//!
//! 包含网络接口信息、手动连接请求等数据结构

use serde::{Deserialize, Serialize};

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// 接口名称 (如 "en0", "Wi-Fi", "以太网")
    pub name: String,
    /// IP 地址
    pub ip: String,
    /// 是否为回环地址
    pub is_loopback: bool,
    /// 是否为 IPv4
    pub is_ipv4: bool,
}

/// 手动连接请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualConnectionRequest {
    /// 目标设备 IP 地址
    pub ip: String,
    /// 目标设备端口
    pub port: u16,
}

/// 手动连接响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualConnectionResponse {
    /// 是否成功
    pub success: bool,
    /// 设备 ID（连接成功时返回）
    pub device_id: Option<String>,
    /// 响应消息
    pub message: String,
}

/// 连接请求信息（发送给接收方）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestMessage {
    /// 请求方设备 ID
    pub requester_device_id: String,
    /// 请求方 IP 地址
    pub requester_ip: String,
    /// 请求方设备别名（可选）
    pub requester_alias: Option<String>,
    /// 请求方平台（可选）
    pub requester_platform: Option<String>,
}

/// 连接响应信息（接收方返回给发起方）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponseMessage {
    /// 是否接受连接
    pub accepted: bool,
    /// 响应方设备 ID
    pub responder_device_id: String,
    /// 响应方 IP 地址（可选）
    pub responder_ip: Option<String>,
    /// 响应方设备别名（可选）
    pub responder_alias: Option<String>,
}

/// 连接请求决策（前端用户确认）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestDecision {
    /// 是否接受连接
    pub accept: bool,
    /// 请求方设备 ID
    pub requester_device_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_interface_serialization() {
        let iface = NetworkInterface {
            name: "en0".to_string(),
            ip: "192.168.1.100".to_string(),
            is_loopback: false,
            is_ipv4: true,
        };

        let json = serde_json::to_string(&iface).unwrap();
        let deserialized: NetworkInterface = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "en0");
        assert_eq!(deserialized.ip, "192.168.1.100");
        assert!(!deserialized.is_loopback);
        assert!(deserialized.is_ipv4);
    }

    #[test]
    fn test_manual_connection_request() {
        let request = ManualConnectionRequest {
            ip: "192.168.1.100".to_string(),
            port: 29217,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ManualConnectionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.ip, "192.168.1.100");
        assert_eq!(deserialized.port, 29217);
    }

    #[test]
    fn test_connection_request_message() {
        let msg = ConnectionRequestMessage {
            requester_device_id: "123456".to_string(),
            requester_ip: "192.168.1.100".to_string(),
            requester_alias: Some("My Device".to_string()),
            requester_platform: Some("macos aarch64".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ConnectionRequestMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.requester_device_id, "123456");
        assert_eq!(deserialized.requester_ip, "192.168.1.100");
        assert_eq!(deserialized.requester_alias, Some("My Device".to_string()));
        assert_eq!(
            deserialized.requester_platform,
            Some("macos aarch64".to_string())
        );
    }
}
