//! Use case for getting application settings
//! 获取应用设置的用例

use anyhow::Result;
use tracing::{info_span, info, Instrument};
use uc_core::ports::settings::SettingsPort;
use uc_core::settings::model::Settings;

/// Use case for retrieving application settings.
///
/// ## Behavior / 行为
/// - Loads settings from the settings port
/// - Returns the complete settings structure
///
/// ## English
/// Loads the current application settings from the configured
/// settings repository and returns them to the caller.
pub struct GetSettings {
    settings: std::sync::Arc<dyn SettingsPort>,
}

impl GetSettings {
    /// Create a new GetSettings use case.
    pub fn new(settings: std::sync::Arc<dyn SettingsPort>) -> Self {
        Self { settings }
    }

    /// Execute the use case.
    ///
    /// # Returns / 返回值
    /// - `Ok(Settings)` - The current application settings
    /// - `Err(e)` if loading settings fails
    pub async fn execute(&self) -> Result<Settings> {
        let span = info_span!("usecase.get_settings.execute");

        async {
            info!("Retrieving application settings");

            let result = self.settings.load().await?;

            info!("Settings retrieved successfully");
            Ok(result)
        }
        .instrument(span)
        .await
    }
}
