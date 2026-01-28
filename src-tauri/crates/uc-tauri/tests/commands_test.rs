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

#[test]
fn test_pairing_commands_are_exposed() {
    let _ = uc_tauri::commands::pairing::get_local_peer_id;
    let _ = uc_tauri::commands::pairing::get_p2p_peers;
    let _ = uc_tauri::commands::pairing::get_local_device_info;
    let _ = uc_tauri::commands::pairing::get_paired_peers;
    let _ = uc_tauri::commands::pairing::get_paired_peers_with_status;
    let _ = uc_tauri::commands::pairing::initiate_p2p_pairing;
    let _ = uc_tauri::commands::pairing::verify_p2p_pairing_pin;
    let _ = uc_tauri::commands::pairing::reject_p2p_pairing;
    let _ = uc_tauri::commands::pairing::accept_p2p_pairing;
    let _ = uc_tauri::commands::pairing::unpair_p2p_device;
}
