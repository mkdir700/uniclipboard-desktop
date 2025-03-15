use uniclipboard::Args;
use clap::Parser;

#[test]
fn test_parse_args_with_all_options() {
    let args = Args::parse_from([
        "test",
        "--webdav-url", "https://example.com/webdav",
        "--webdav-username", "testuser",
        "--webdav-password", "testpass",
        "--interactive",
        "--server",
        "--server-ip", "192.168.1.100",
        "--server-port", "8080"
    ]);

    assert_eq!(args.webdav_url, Some("https://example.com/webdav".to_string()));
    assert_eq!(args.webdav_username, Some("testuser".to_string()));
    assert_eq!(args.webdav_password, Some("testpass".to_string()));
    assert!(args.interactive);
    assert!(args.server);
    assert_eq!(args.server_ip, Some("192.168.1.100".to_string()));
    assert_eq!(args.server_port, Some(8080));
}

#[test]
fn test_parse_args_with_short_options() {
    let args = Args::parse_from([
        "test",
        "--server",
        "--server-ip", "192.168.1.100",
        "--server-port", "8080"
    ]);

    assert!(args.server);
    assert_eq!(args.server_ip, Some("192.168.1.100".to_string()));
    assert_eq!(args.server_port, Some(8080));
}

#[test]
fn test_parse_args_with_no_options() {
    let args = Args::parse_from(["test"]);

    assert_eq!(args.webdav_url, None);
    assert_eq!(args.webdav_username, None);
    assert_eq!(args.webdav_password, None);
    assert!(!args.interactive);
    assert!(!args.server);
    assert_eq!(args.server_ip, None);
    assert_eq!(args.server_port, None);
}

#[test]
fn test_parse_args_with_interactive_only() {
    let args = Args::parse_from(["test", "--interactive"]);

    assert!(args.interactive);
    assert_eq!(args.webdav_url, None);
    assert_eq!(args.webdav_username, None);
    assert_eq!(args.webdav_password, None);
    assert!(!args.server);
    assert_eq!(args.server_ip, None);
    assert_eq!(args.server_port, None);
}

#[test]
fn test_parse_args_with_server_only() {
    let args = Args::parse_from(["test", "--server"]);

    assert!(args.server);
    assert!(!args.interactive);
    assert_eq!(args.webdav_url, None);
    assert_eq!(args.webdav_username, None);
    assert_eq!(args.webdav_password, None);
    assert_eq!(args.server_ip, None);
    assert_eq!(args.server_port, None);
}

#[test]
fn test_parse_args_with_webdav_only() {
    let args = Args::parse_from([
        "test",
        "--webdav-url", "https://example.com/webdav",
        "--webdav-username", "testuser",
        "--webdav-password", "testpass"
    ]);

    assert_eq!(args.webdav_url, Some("https://example.com/webdav".to_string()));
    assert_eq!(args.webdav_username, Some("testuser".to_string()));
    assert_eq!(args.webdav_password, Some("testpass".to_string()));
    assert!(!args.interactive);
    assert!(!args.server);
    assert_eq!(args.server_ip, None);
    assert_eq!(args.server_port, None);
}

// 移除这个辅助函数，因为我们不再需要它
// fn parse_args_from(args: Vec<String>) -> Args {
//     Args::parse_from(args)
// }
