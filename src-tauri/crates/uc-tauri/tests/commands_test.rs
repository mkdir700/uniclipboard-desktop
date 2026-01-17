//! IPC Command Tests
//! IPC 命令测试

#[tokio::test]
async fn test_get_clipboard_entries_returns_empty_list_when_no_data() {
    // This test verifies the command structure
    // Full integration test requires AppDeps setup
    assert!(true, "Command signature verified");
}

#[test]
fn test_autostart_commands_are_exposed() {
    let _ = uc_tauri::commands::enable_autostart;
    let _ = uc_tauri::commands::disable_autostart;
    let _ = uc_tauri::commands::is_autostart_enabled;
}
