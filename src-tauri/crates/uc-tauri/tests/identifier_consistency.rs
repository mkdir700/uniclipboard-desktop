use std::fs;
use std::path::PathBuf;

#[test]
fn identifier_matches_uniclipboard() {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", config_path, e));
    let json: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", config_path, e));

    let identifier = json
        .get("identifier")
        .and_then(|value| value.as_str())
        .unwrap_or("<missing>");

    assert_eq!(identifier, "uniclipboard");
}
