use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{debug, warn};
use uc_core::{ports::TimerPort, SessionId};

pub struct Timer {
    timers: Arc<Mutex<HashMap<SessionId, tokio::task::AbortHandle>>>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl TimerPort for Timer {
    async fn start(
        &mut self,
        session_id: &SessionId,
        ttl_secs: u64,
    ) -> anyhow::Result<oneshot::Receiver<SessionId>> {
        let (tx, rx) = oneshot::channel();
        let timers = Arc::clone(&self.timers);
        let session_id_clone = session_id.clone();

        let mut timers_guard = self.timers.lock().await;
        if let Some(existing) = timers_guard.remove(session_id) {
            existing.abort();
        }

        let handle = tokio::spawn(async move {
            sleep(Duration::from_secs(ttl_secs)).await;
            if tx.send(session_id_clone.clone()).is_err() {
                warn!(session_id = %session_id_clone, "timer timeout receiver dropped");
            }

            let mut timers_guard = timers.lock().await;
            timers_guard.remove(&session_id_clone);
        });

        timers_guard.insert(session_id.clone(), handle.abort_handle());
        debug!(session_id = %session_id, ttl_secs, "timer started");
        Ok(rx)
    }

    async fn stop(&mut self, session_id: &SessionId) -> anyhow::Result<()> {
        let mut timers_guard = self.timers.lock().await;
        if let Some(handle) = timers_guard.remove(session_id) {
            handle.abort();
            debug!(session_id = %session_id, "timer stopped");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{advance, Duration};

    #[tokio::test]
    async fn start_sends_timeout_after_ttl() -> anyhow::Result<()> {
        tokio::time::pause();
        let mut timer = Timer::new();
        let session_id = SessionId::from("session-1");

        let rx = timer.start(&session_id, 5).await?;
        advance(Duration::from_secs(5)).await;

        let received = rx.await?;
        assert_eq!(received, session_id);
        Ok(())
    }

    #[tokio::test]
    async fn stop_cancels_timer() -> anyhow::Result<()> {
        tokio::time::pause();
        let mut timer = Timer::new();
        let session_id = SessionId::from("session-2");

        let rx = timer.start(&session_id, 5).await?;
        timer.stop(&session_id).await?;
        advance(Duration::from_secs(10)).await;

        assert!(rx.await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn start_replaces_existing_timer_for_same_session() -> anyhow::Result<()> {
        tokio::time::pause();
        let mut timer = Timer::new();
        let session_id = SessionId::from("session-3");

        let rx = timer.start(&session_id, 5).await?;
        let rx2 = timer.start(&session_id, 10).await?;
        advance(Duration::from_secs(5)).await;

        assert!(rx.await.is_err());

        advance(Duration::from_secs(5)).await;
        let received = rx2.await?;
        assert_eq!(received, session_id);
        Ok(())
    }
}
