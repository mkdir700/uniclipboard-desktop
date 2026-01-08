use std::sync::Arc;

use super::event_bus::{PlatformCommandReceiver, PlatformEventReceiver};
use crate::clipboard::watcher::ClipboardWatcher;
use crate::clipboard::LocalClipboard;
use crate::ipc::{PlatformCommand, PlatformEvent};
use crate::ports::{LocalClipboardPort, PlatformCommandExecutorPort};
use anyhow::Result;
use clipboard_rs::ClipboardWatcherContext;
use log::error;

pub struct PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    local_clipboard: Arc<dyn LocalClipboardPort>,
    event_rx: PlatformEventReceiver,
    command_rx: PlatformCommandReceiver,
    executor: Arc<E>,
    shutting_down: bool,
    watcher_join: Option<JoinHandle<()>>,
    watcher_handle: Option<Arc<ClipboardWatcher<dyn LocalClipboardPort>>>,
}

impl<E> PlatformRuntime<E>
where
    E: PlatformCommandExecutorPort,
{
    pub fn new(
        event_rx: PlatformEventReceiver,
        command_rx: PlatformCommandReceiver,
        executor: Arc<E>,
    ) -> Result<PlatformRuntime<E>, anyhow::Error> {
        let local_clipboard = Arc::new(LocalClipboard::new()?);

        Ok(Self {
            local_clipboard,
            event_rx,
            command_rx,
            executor,
            shutting_down: false,
            watcher_join: None,
            watcher_handle: None,
        })
    }

    pub async fn start(mut self) {
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

    fn start_clipboard_watcher(&mut self) -> Result<()> {
        let mut watcher_ctx = ClipboardWatcherContext::new()?;

        let handler = ClipboardWatcher::new(self.local_clipboard.clone(), self.event_rx.clone());

        let shutdown = watcher_ctx.add_handler(handler).get_shutdown_channel();

        let join = tokio::task::spawn_blocking(move || {
            log::info!("start clipboard watch");
            watcher_ctx.start_watch();
            log::info!("clipboard watch stopped");
        });

        self.watcher_join = Some(join);
        self.watcher_stop = Some(shutdown);
        Ok(())
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
            PlatformCommand::ReadClipboard => {
                todo!()
            }
            PlatformCommand::WriteClipboard => {
                todo!()
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
