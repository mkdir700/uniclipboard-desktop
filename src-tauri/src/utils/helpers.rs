use local_ip_address::{local_ip, list_afinet_netifas};
use sha2::{Digest, Sha256};

use crate::domain::network::NetworkInterface;

pub fn string_to_32_bytes(input: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

pub fn generate_device_id() -> String {
    // 生成6位随机数字
    let random_number = rand::random::<u32>() % 1000000;
    format!("{:06}", random_number)
}

/// 检查 IP 地址是否有效
pub fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    for part in parts {
        if part.parse::<u8>().is_err() {
            return false;
        }
    }
    true
}

/// 检查端口是否有效
pub fn is_valid_port(port: u16) -> bool {
    port >= 1024
}

/// 获取以太网 IP 地址或 WiFi IP 地址
pub fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(e) => {
            // 记录错误并返回一个默认值
            eprintln!("获取本地 IP 地址时出错: {}", e);
            "127.0.0.1".to_string()
        }
    }
}

/// 获取所有网络接口的 IP 地址
///
/// 返回所有非回环的 IPv4 地址，用于显示本机可用的网络接口
pub fn get_local_network_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    match list_afinet_netifas() {
        Ok(network_interfaces) => {
            for (name, ip) in network_interfaces {
                let is_loopback = ip.is_loopback();
                let is_ipv4 = matches!(ip, std::net::IpAddr::V4(_));

                // 过滤掉 IPv6 和回环地址
                if is_ipv4 && !is_loopback {
                    interfaces.push(NetworkInterface {
                        name,
                        ip: ip.to_string(),
                        is_loopback,
                        is_ipv4,
                    });
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get network interfaces: {}", e);
        }
    }

    interfaces
}

/// 当前时间
pub fn get_current_time() -> i32 {
    chrono::Utc::now().timestamp() as i32
}

/// 获取当前平台
pub fn get_current_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("{} {}", os, arch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_32_bytes() {
        let input = "test";
        let output = string_to_32_bytes(input);
        assert_eq!(output.len(), 32);
    }

    #[test]
    fn test_generate_device_id() {
        let id = generate_device_id();
        assert_eq!(id.len(), 6);
    }

    #[test]
    fn test_is_valid_ip() {
        assert!(is_valid_ip("192.168.1.1"));
        assert!(is_valid_ip("0.0.0.0"));
        assert!(is_valid_ip("255.255.255.255"));
        assert!(!is_valid_ip("256.256.256.256"));
        assert!(!is_valid_ip("192.168.1"));
        assert!(!is_valid_ip("192.168.1.1.1"));
        assert!(!is_valid_ip("192.168.1.a"));
        assert!(!is_valid_ip("..."));
    }

    #[test]
    fn test_is_valid_port() {
        assert!(!is_valid_port(0));
        assert!(!is_valid_port(1023));
        assert!(is_valid_port(1024));
        assert!(is_valid_port(8080));
        assert!(is_valid_port(65535));
    }

    #[test]
    fn test_get_local_ip() {
        let ip = get_local_ip();
        println!("local ip: {}", ip);
        assert!(is_valid_ip(&ip));
    }

    #[test]
    fn test_get_local_network_interfaces() {
        let interfaces = get_local_network_interfaces();
        println!("Found {} network interfaces:", interfaces.len());
        for iface in &interfaces {
            println!("  - {}: {}", iface.name, iface.ip);
        }

        // 至少应该有一个非回环的 IPv4 地址（如果设备有网络连接）
        // 但在某些环境下（如 CI），可能没有网络接口
        for iface in &interfaces {
            assert!(is_valid_ip(&iface.ip), "Invalid IP: {}", iface.ip);
            assert!(!iface.is_loopback, "Loopback address should be filtered");
            assert!(iface.is_ipv4, "Only IPv4 addresses should be included");
        }
    }
}
