use anyhow::Result;
use std::sync::Arc;
use uc_core::{
    ports::SettingsPort,
    settings::model::Theme,
};

pub struct ApplyThemeSetting<S>
where
    S: SettingsPort,
{
    settings: Arc<S>,
}

impl<S> ApplyThemeSetting<S>
where
    S: SettingsPort,
{
    pub async fn execute(&self, theme: Theme) -> Result<()> {
        let mut settings = self.settings.load().await?;
        if settings.general.theme == theme {
            return Ok(());
        }

        self.settings.save(&settings).await?;
        settings.general.theme = theme;

        Ok(())
    }
}
