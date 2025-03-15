use serial_test::serial;
use std::env;
use std::fs;
use tempfile::tempdir;
use uniclipboard::config::{get_config_path, Config};

#[test]
#[serial]
fn test_load_config() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("uniclipboard");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    let config_content = r#"
        device_id = "test_device"
        device_name = "测试设备"
        webdav_url = "https://example.com/webdav"
        username = "testuser"
        password = "testpass"
        enable_webdav = true
        push_interval = 1000
        pull_interval = 1000
        sync_interval = 1000
        enable_push = true
        enable_pull = true
        enable_key_mouse_monitor = true
        key_mouse_monitor_sleep_timeout = 6000
        enable_websocket = true
        is_server = false
        webserver_addr = "127.0.0.1"
        webserver_port = 8114
        peer_device_addr = "127.0.0.1"
        peer_device_port = 8116
    "#;
    fs::write(&config_path, config_content).unwrap();

    env::set_var("UNICLIPBOARD_CONFIG_PATH", config_path.to_str().unwrap());

    let config = Config::load(Some(config_path.clone())).unwrap();
    assert_eq!(config.device_id, "test_device");
    assert_eq!(config.device_name, Some("测试设备".to_string()));
    assert_eq!(config.webdav_url, Some("https://example.com/webdav".to_string()));
    assert_eq!(config.username, Some("testuser".to_string()));
    assert_eq!(config.password, Some("testpass".to_string()));
    assert_eq!(config.enable_webdav, Some(true));
    assert_eq!(config.push_interval, Some(1000));
    assert_eq!(config.pull_interval, Some(1000));
    assert_eq!(config.sync_interval, Some(1000));
    assert_eq!(config.enable_push, Some(true));
    assert_eq!(config.enable_pull, Some(true));
    assert_eq!(config.enable_key_mouse_monitor, Some(true));
    assert_eq!(config.key_mouse_monitor_sleep_timeout, Some(6000));
    assert_eq!(config.enable_websocket, Some(true));
    assert_eq!(config.webserver_addr, Some("127.0.0.1".to_string()));
    assert_eq!(config.webserver_port, Some(8114));
    assert_eq!(config.peer_device_addr, Some("127.0.0.1".to_string()));
    assert_eq!(config.peer_device_port, Some(8116));
}

#[test]
#[serial]
fn test_save_config() {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("uniclipboard");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");

    env::set_var("UNICLIPBOARD_CONFIG_PATH", config_path.to_str().unwrap());

    let mut config = Config::default();
    config.device_id = "save_test_device".to_string();
    config.device_name = Some("保存测试设备".to_string());
    config.webdav_url = Some("https://save.example.com/webdav".to_string());
    config.username = Some("save_testuser".to_string());
    config.password = Some("save_testpass".to_string());
    config.enable_webdav = Some(true);
    config.push_interval = Some(2000);
    config.pull_interval = Some(2000);
    config.sync_interval = Some(2000);
    config.enable_push = Some(false);
    config.enable_pull = Some(false);
    config.enable_key_mouse_monitor = Some(false);
    config.key_mouse_monitor_sleep_timeout = Some(7000);
    config.enable_websocket = Some(false);
    config.webserver_addr = Some("192.168.1.100".to_string());
    config.webserver_port = Some(8116);

    config.save(Some(config_path.clone())).unwrap();

    assert!(config_path.exists());

    let saved_config = Config::load(Some(config_path.clone())).unwrap();
    assert_eq!(saved_config.device_id, "save_test_device");
    assert_eq!(saved_config.device_name, Some("保存测试设备".to_string()));
    assert_eq!(saved_config.webdav_url, Some("https://save.example.com/webdav".to_string()));
    assert_eq!(saved_config.username, Some("save_testuser".to_string()));
    assert_eq!(saved_config.password, Some("save_testpass".to_string()));
    assert_eq!(saved_config.enable_webdav, Some(true));
    assert_eq!(saved_config.push_interval, Some(2000));
    assert_eq!(saved_config.pull_interval, Some(2000));
    assert_eq!(saved_config.sync_interval, Some(2000));
    assert_eq!(saved_config.enable_push, Some(false));
    assert_eq!(saved_config.enable_pull, Some(false));
    assert_eq!(saved_config.enable_key_mouse_monitor, Some(false));
    assert_eq!(saved_config.key_mouse_monitor_sleep_timeout, Some(7000));
    assert_eq!(saved_config.enable_websocket, Some(false));
    assert_eq!(saved_config.webserver_addr, Some("192.168.1.100".to_string()));
    assert_eq!(saved_config.webserver_port, Some(8116));
}

#[test]
fn test_get_config_path() {
    let temp_dir = tempdir().unwrap();

    // 根据不同的操作系统设置适当的环境变量
    #[cfg(target_os = "windows")]
    {
        env::set_var("USERPROFILE", temp_dir.path());
        // !无法模拟 FOLDERID_RoamingAppData，使用环境变量代替
        env::set_var(
            "UNICLIPBOARD_CONFIG_PATH",
            temp_dir.path().join("uniclipboard").join("config.toml"),
        );
    }
    #[cfg(target_os = "macos")]
    {
        env::set_var("HOME", temp_dir.path());
        // 清除可能影响测试的环境变量
        env::remove_var("UNICLIPBOARD_CONFIG_PATH");
    }
    #[cfg(target_os = "linux")]
    {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        // 清除可能影响测试的环境变量
        env::remove_var("UNICLIPBOARD_CONFIG_PATH");
    }

    let config_path = get_config_path().unwrap();

    // 构建预期的配置路径
    let expected_path = if cfg!(target_os = "windows") {
        temp_dir.path().join("uniclipboard").join("config.toml")
    } else if cfg!(target_os = "macos") {
        temp_dir
            .path()
            .join("Library")
            .join("Application Support")
            .join("uniclipboard")
            .join("config.toml")
    } else {
        temp_dir.path().join("uniclipboard").join("config.toml")
    };

    assert_eq!(
        config_path,
        expected_path,
        "Config path mismatch on {:?}",
        std::env::consts::OS
    );
}
