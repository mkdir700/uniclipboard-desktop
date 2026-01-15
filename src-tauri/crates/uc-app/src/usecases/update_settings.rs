//! Use case for updating application settings
//! 更新应用设置的用例

use anyhow::Result;
use tracing::{info_span, info, Instrument};
use uc_core::ports::SettingsPort;
use uc_core::settings::model::Settings;

/// Use case for updating application settings.
///
/// ## Behavior / 行为
/// - Validates settings (basic validation)
/// - Persists settings through the settings port
///
/// ## English
/// Updates the application settings by validating and persisting
/// the provided settings through the configured settings repository.
pub struct UpdateSettings {
    settings: std::sync::Arc<dyn SettingsPort>,
}

impl UpdateSettings {
    /// Create a new UpdateSettings use case.
    pub fn new(settings: std::sync::Arc<dyn SettingsPort>) -> Self {
        Self { settings }
    }

    /// Execute the use case.
    ///
    /// # Parameters / 参数
    /// - `settings`: The settings to persist
    ///
    /// # Returns / 返回值
    /// - `Ok(())` if settings are saved successfully
    /// - `Err(e)` if validation or save fails
    pub async fn execute(&self, settings: Settings) -> Result<()> {
        let span = info_span!("usecase.update_settings.execute");

        async {
            info!("Updating application settings");

            // Basic validation: ensure schema version is current
            let current_version = uc_core::settings::model::CURRENT_SCHEMA_VERSION;
            if settings.schema_version != current_version {
                return Err(anyhow::anyhow!(
                    "Invalid schema version: expected {}, got {}",
                    current_version,
                    settings.schema_version
                ));
            }

            // Persist settings
            self.settings.save(&settings).await?;

            info!("Settings updated successfully");
            Ok(())
        }
        .instrument(span)
        .await
    }
}
