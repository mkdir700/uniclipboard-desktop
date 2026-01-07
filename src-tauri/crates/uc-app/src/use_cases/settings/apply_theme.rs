use anyhow::Result;
use std::sync::Arc;
use uc_core::{
    ports::{AutostartPort, SettingsPort},
    settings::model::Theme,
};

pub struct ApplyThemeSetting<S, A>
where
    S: SettingsPort,
    A: AutostartPort,
{
    settings: Arc<S>,
    autostart: Arc<A>,
}

impl<S, A> ApplyThemeSetting<S, A>
where
    S: SettingsPort,
    A: AutostartPort,
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
