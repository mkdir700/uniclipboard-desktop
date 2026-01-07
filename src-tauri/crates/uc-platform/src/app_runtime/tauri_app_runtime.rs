use async_trait::async_trait;
use uc_app::AppEvent;
use uc_core::ports::AppRuntimePort;

use crate::ports::app_event_handler::AppEventHandlerPort;

pub struct TauriAppRuntime {
    pub handlers: Vec<Box<dyn AppEventHandlerPort<AppEvent>>>,
}

impl TauriAppRuntime {
    pub fn new(handlers: Vec<Box<dyn AppEventHandlerPort<AppEvent>>>) -> Self {
        Self { handlers }
    }
}

#[async_trait]
impl AppRuntimePort<AppEvent> for TauriAppRuntime {
    async fn emit(&self, event: AppEvent) {
        for handler in &self.handlers {
            handler.handle(event.clone()).await;
        }
    }

    async fn exit(&self) {
        std::process::exit(0);
    }
}
