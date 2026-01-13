use anyhow::Result;
use std::sync::Arc;
use uc_core::ports::UiPort;

pub struct OpenSettingsWindow<A>
where
    A: UiPort,
{
    ui: Arc<A>,
}

impl<A> OpenSettingsWindow<A>
where
    A: UiPort,
{
    pub async fn execute(&self) -> Result<()> {
        self.ui.open_settings().await
    }
}
