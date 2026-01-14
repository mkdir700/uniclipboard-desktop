//! Integration tests for settings use cases
//!
//! Tests the complete flow from command to persistence

use std::sync::Arc;
use tempfile::tempdir;
use uc_app::usecases::{GetSettings, UpdateSettings};
use uc_core::settings::model::{Settings, CURRENT_SCHEMA_VERSION};
use uc_infra::settings::repository::FileSettingsRepository;

#[tokio::test]
async fn test_get_settings_returns_defaults() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path);
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    let uc = GetSettings::new(repo_arc.clone());
    let settings = uc.execute().await.unwrap();

    // Should return defaults since file doesn't exist
    assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
}

#[tokio::test]
async fn test_update_settings_persists() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path.clone());
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    // Update settings
    let mut settings = Settings::default();
    settings.general.device_name = Some("test_device".to_string());

    let update_uc = UpdateSettings::new(repo_arc.clone());
    update_uc.execute(settings.clone()).await.unwrap();

    // Verify persistence through GetSettings
    let get_uc = GetSettings::new(repo_arc);
    let loaded = get_uc.execute().await.unwrap();

    assert_eq!(loaded.general.device_name, Some("test_device".to_string()));
}

#[tokio::test]
async fn test_update_settings_validates_schema_version() {
    let temp_dir = tempdir().unwrap();
    let settings_path = temp_dir.path().join("test_settings.json");

    let repo = FileSettingsRepository::new(settings_path);
    let repo_arc: Arc<dyn uc_core::ports::settings::SettingsPort> = Arc::new(repo);

    let mut settings = Settings::default();
    settings.schema_version = 999; // Invalid version

    let update_uc = UpdateSettings::new(repo_arc);
    let result = update_uc.execute(settings).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid schema version"));
}
