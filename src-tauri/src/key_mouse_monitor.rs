use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use std::thread;

use async_trait::async_trait;


#[async_trait]
pub trait KeyMouseMonitorTrait: Send + Sync {
    async fn start(&self);
    async fn is_sleep(&self) -> bool;
}

pub struct KeyMouseMonitor {
    last_activity: Arc<Mutex<Instant>>,
    sleep_timeout: Duration,
    pub is_running: Arc<Mutex<bool>>,
    state_receiver: Arc<Mutex<mpsc::Receiver<(MouseState, Vec<Keycode>)>>>,
}

impl KeyMouseMonitor {
    #[allow(dead_code)]
    pub fn new(sleep_timeout: Duration) -> Self {
        let (state_sender, state_receiver) = mpsc::channel(100);
        let is_running = Arc::new(Mutex::new(false));
        
        let is_running_clone = is_running.clone();
        thread::Builder::new()
            .name("device_state_poller".to_string())
            .spawn(move || {
                let device_state = DeviceState::new();
                while *is_running_clone.blocking_lock() {
                    let mouse = device_state.get_mouse();
                    let keys = device_state.get_keys();
                    if state_sender.blocking_send((mouse, keys)).is_err() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            })
            .expect("Failed to spawn device state poller thread");

        Self {
            last_activity: Arc::new(Mutex::new(Instant::now())),
            sleep_timeout,
            is_running,
            state_receiver: Arc::new(Mutex::new(state_receiver)),
        }
    }

    /// 获取上次活动时间
    pub async fn get_last_activity(&self) -> Instant {
        *self.last_activity.lock().await
    }

    /// 更新上次活动时间
    #[allow(dead_code)]
    pub async fn update_last_activity(&self) {
        let mut last_activity = self.last_activity.lock().await;
        *last_activity = Instant::now();
    }

    /// 停止监控
    ///
    /// 将 is_running 设置为 false，停止监控。
    #[allow(dead_code)]
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
    }

    /// 设置休眠状态, 仅用于测试
    ///
    /// 如果休眠状态为 true，则将 is_running 设置为 false，停止监控。
    /// 如果休眠状态为 false，则将 is_running 设置为 true，开始监控。
    #[cfg(feature = "testing")]
    #[allow(dead_code)]
    pub async fn set_sleep(&self, value: bool) {
        let mut is_running = self.is_running.lock().await;
        *is_running = value;
    }
}

#[async_trait]
impl KeyMouseMonitorTrait for KeyMouseMonitor {
    /// 是否休眠
    ///
    /// 如果上次活动时间与当前时间的时间差大于休眠超时时间，则返回 true，否则返回 false。
    async fn is_sleep(&self) -> bool {
        let last_activity = self.get_last_activity().await;
        let now = Instant::now();
        now.duration_since(last_activity) > self.sleep_timeout
    }

    /// 开始监控
    ///
    /// 如果已经正在监控，则直接返回。
    /// 否则，将 is_running 设置为 true，并启动一个异步任务来监控键鼠活动。
    /// 在监控过程中，会不断检查键鼠活动是否发生变化，如果发生变化，则更新上次活动时间。
    /// 监控过程中，每隔 200 毫秒检查一次键鼠活动。
    /// 如果键鼠活动发生变化，则更新上次活动时间。
    /// 如果键鼠活动长时间没有变化，则认为系统处于休眠状态。
    async fn start(&self) {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return;
        }
        *is_running = true;
        drop(is_running);

        let is_running = self.is_running.clone();
        let last_activity = self.last_activity.clone();
        let state_receiver = self.state_receiver.clone();

        tokio::spawn(async move {
            let mut last_mouse = MouseState::default();
            let mut last_keys = Vec::new();

            while *is_running.lock().await {
                if let Ok((current_mouse, current_keys)) = state_receiver.lock().await.try_recv() {
                    if last_mouse != current_mouse || last_keys != current_keys {
                        let mut last_activity = last_activity.lock().await;
                        *last_activity = Instant::now();
                    }

                    last_mouse = current_mouse;
                    last_keys = current_keys;
                }

                sleep(Duration::from_millis(10)).await;
            }
        });
    }

}

#[cfg(test)]
#[tokio::test]
async fn test_new_monitor() {
    let timeout = Duration::from_secs(10);
    let monitor = KeyMouseMonitor::new(timeout);
    assert_eq!(monitor.sleep_timeout, timeout);
}
