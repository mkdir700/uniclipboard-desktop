use super::clipboard::{RsClipboard, RsClipboardChangeHandler};
use crate::config::Setting;
use crate::interface::local_clipboard_trait::LocalClipboardTrait;
use crate::message::Payload;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::{ClipboardWatcher, ClipboardWatcherContext};
use log::error;
use log::{debug, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::{mpsc, Notify};
use tokio::sync::{Mutex as TokioMutex, RwLock};
use tokio::time::{Duration, Instant};

#[derive(Clone)]
pub struct LocalClipboard {
    rs_clipboard: Arc<RsClipboard>,
    paused: Arc<(TokioMutex<bool>, Notify)>,
    stopped: Arc<TokioMutex<bool>>,
    watcher: Arc<Mutex<ClipboardWatcherContext<RsClipboardChangeHandler>>>,
    rw_lock: Arc<RwLock<bool>>,
    last_write: Arc<TokioMutex<Instant>>,
    write_cooldown: Duration,
    is_self_write: Arc<AtomicBool>,
}

impl LocalClipboard {
    /// 创建一个新的 LocalClipboard 实例，使用指定的配置
    pub fn with_user_setting(user_setting: Setting) -> Result<Self> {
        let notify = Arc::new(Notify::new());
        let rs_clipboard = RsClipboard::with_user_setting(notify.clone(), user_setting)?;
        let mut watcher: ClipboardWatcherContext<RsClipboardChangeHandler> =
            ClipboardWatcherContext::new()
                .map_err(|e| anyhow!("Failed to create clipboard watcher context: {:?}", e))?;

        // 注册剪切板变化处理器，这是剪切板监听能够工作的关键
        let clipboard_change_handler = RsClipboardChangeHandler::new(notify.clone());
        watcher.add_handler(clipboard_change_handler);
        info!("Clipboard watcher handler registered successfully");

        Ok(Self {
            rs_clipboard: Arc::new(rs_clipboard),
            paused: Arc::new((TokioMutex::new(false), Notify::new())),
            stopped: Arc::new(TokioMutex::new(false)),
            watcher: Arc::new(Mutex::new(watcher)),
            rw_lock: Arc::new(RwLock::new(false)),
            last_write: Arc::new(TokioMutex::new(Instant::now())),
            write_cooldown: Duration::from_millis(500), // 在 500ms 内，忽略自己写入导致的剪贴板变更事件
            is_self_write: Arc::new(AtomicBool::new(false)),
        })
    }
}

#[async_trait]
impl LocalClipboardTrait for LocalClipboard {
    async fn read(&self) -> Result<Payload> {
        let reading_lock = self.rw_lock.read().await;
        let payload = self.rs_clipboard.read()?;
        drop(reading_lock);
        Ok(payload)
    }

    async fn write(&self, payload: Payload) -> Result<()> {
        let writing_lock = self.rw_lock.write().await;
        self.rs_clipboard.write(payload)?;
        drop(writing_lock);
        Ok(())
    }

    async fn start_monitoring(&self) -> Result<mpsc::Receiver<Payload>> {
        info!("Starting local clipboard monitoring");
        let (tx, rx) = mpsc::channel(100);
        let rs_clipboard = Arc::clone(&self.rs_clipboard);
        let watcher = Arc::clone(&self.watcher);
        let stopped = Arc::clone(&self.stopped);
        let paused = Arc::clone(&self.paused);
        let self_clone = self.clone();

        // 在后台线程中启动剪贴板监听
        thread::spawn(move || {
            let mut watcher = match watcher.lock() {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to lock clipboard watcher: {}", e);
                    return;
                }
            };
            info!("Clipboard watcher thread started, calling start_watch()");
            watcher.start_watch();
        });

        // 在异步任务中处理剪贴板变化
        tokio::spawn(async move {
            info!("Clipboard change handler task started, waiting for changes");
            loop {
                if *stopped.lock().await {
                    break;
                }

                if *paused.0.lock().await {
                    paused.1.notified().await;
                    continue;
                }

                if let Err(e) = rs_clipboard.wait_clipboard_change().await {
                    error!("Wait clipboard change failed: {:?}", e);
                    continue;
                }

                info!("Clipboard change detected, processing new content");

                // 如果是因为刚写入剪切板导致的剪切板变更事件，则跳过本次
                let now = Instant::now();
                let last_write = *self_clone.last_write.lock().await;
                if now.duration_since(last_write) < self_clone.write_cooldown {
                    if self_clone.is_self_write.load(Ordering::SeqCst) {
                        info!("Skip clipboard change event because of self write");
                        self_clone.is_self_write.store(false, Ordering::SeqCst);
                        continue;
                    }
                }

                if *paused.0.lock().await {
                    paused.1.notified().await;
                    continue;
                }

                match self_clone.read().await {
                    Ok(payload) => {
                        info!("Clipboard content read successfully: {}, sending to sync service", payload);
                        if let Err(e) = tx.send(payload).await {
                            error!("Send payload failed: {:?}", e);
                        } else {
                            info!("Payload sent to sync service successfully");
                        }
                    }
                    Err(e) => {
                        error!("Read clipboard failed: {:?}", e);
                    }
                }
            }
        });
        Ok(rx)
    }

    /// 写入剪贴板内容，并设置自写标志和更新最后写入时间
    async fn set_clipboard_content(&self, content: Payload) -> Result<()> {
        let result = self.write(content).await;
        // 设置自写标志和更新最后写入时间
        self.is_self_write.store(true, Ordering::SeqCst);
        let mut last_write = self.last_write.lock().await;
        *last_write = Instant::now();
        result
    }
}
