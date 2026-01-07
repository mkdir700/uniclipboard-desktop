use crate::event::AppEvent;
use anyhow::Result;
use std::sync::Arc;
use uc_core::ports::AppRuntimePort;

pub struct OpenSettingsWindow<A>
where
    A: AppRuntimePort<AppEvent>,
{
    app_runtime: Arc<A>,
}

impl<A> OpenSettingsWindow<A>
where
    A: AppRuntimePort<AppEvent>,
{
    pub async fn execute(&self) -> Result<()> {
        self.app_runtime.emit(AppEvent::OpenSettingsWindow).await;
        Ok(())
    }
}
