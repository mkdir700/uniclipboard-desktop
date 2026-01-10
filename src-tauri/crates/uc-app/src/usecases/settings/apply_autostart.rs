use anyhow::Result;
use std::sync::Arc;
use uc_core::ports::{AutostartPort, SettingsPort};

pub struct ApplyAutostartSetting<S, A>
where
    S: SettingsPort,
    A: AutostartPort,
{
    settings: Arc<S>,
    autostart: Arc<A>,
}

impl<S, A> ApplyAutostartSetting<S, A>
where
    S: SettingsPort,
    A: AutostartPort,
{
    pub async fn execute(&self, enabled: bool) -> Result<()> {
        let mut settings = self.settings.load().await?;
        if settings.general.auto_start == enabled {
            return Ok(());
        }

        if settings.general.auto_start {
            self.autostart.enable()?;
        } else {
            self.autostart.disable()?;
        }

        settings.general.auto_start = enabled;
        self.settings.save(&settings).await?;

        Ok(())
    }
}
