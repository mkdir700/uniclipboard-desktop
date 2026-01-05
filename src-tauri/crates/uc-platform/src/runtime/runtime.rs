use std::sync::Arc;

use super::event_bus::{CommandReceiver, EventReceiver};
use crate::ipc::{PlatformCommand, PlatformEvent};
use crate::ports::PlatformCommandExecutorPort;
use log::error;

pub struct PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    event_rx: EventReceiver,
    command_rx: CommandReceiver,
    executor: Arc<E>,
    shutting_down: bool,
    // 未来可以加：
    // use_cases: Arc<UseCases>,
}

impl<E> PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    pub fn new(event_rx: EventReceiver, command_rx: CommandReceiver, executor: Arc<E>) -> Self {
        Self {
            event_rx,
            command_rx,
            executor,
            shutting_down: false,
        }
    }
    pub async fn run(mut self) {
        while !self.shutting_down {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    self.handle_event(event).await;
                }
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }

    async fn handle_event(&self, event: PlatformEvent) {
        match event {
            PlatformEvent::ClipboardChanged { content } => {
                // 这里先 log / stub
                // 下一步：交给 SyncClipboard use case
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: PlatformCommand) {
        match command {
            PlatformCommand::Shutdown => {
                self.shutting_down = true;
            }
            other => {
                if let Err(err) = self.executor.execute(other).await {
                    // v1 策略：只记录错误，不崩 runtime
                    error!("Failed to execute platform command: {:?}", err);
                }
            }
        }
    }
}
