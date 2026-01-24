use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uc_core::ports::clipboard::ClipboardChangeOriginPort;
use uc_core::ClipboardChangeOrigin;

pub struct InMemoryClipboardChangeOrigin {
    state: Mutex<Option<OriginState>>,
}

struct OriginState {
    origin: ClipboardChangeOrigin,
    expires_at: Instant,
}

impl InMemoryClipboardChangeOrigin {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }
}

#[async_trait]
impl ClipboardChangeOriginPort for InMemoryClipboardChangeOrigin {
    async fn set_next_origin(&self, origin: ClipboardChangeOrigin, ttl: Duration) {
        let now = Instant::now();
        let expires_at = now.checked_add(ttl).unwrap_or(now);
        let mut state = self.state.lock().await;
        *state = Some(OriginState { origin, expires_at });
    }

    async fn consume_origin_or_default(
        &self,
        default_origin: ClipboardChangeOrigin,
    ) -> ClipboardChangeOrigin {
        let mut state = self.state.lock().await;
        if let Some(stored) = state.take() {
            if Instant::now() <= stored.expires_at {
                return stored.origin;
            }
        }
        default_origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn origin_is_consumed_once() {
        let port = InMemoryClipboardChangeOrigin::new();
        port.set_next_origin(ClipboardChangeOrigin::LocalRestore, Duration::from_secs(1))
            .await;
        let first = port
            .consume_origin_or_default(ClipboardChangeOrigin::LocalCapture)
            .await;
        let second = port
            .consume_origin_or_default(ClipboardChangeOrigin::LocalCapture)
            .await;
        assert_eq!(first, ClipboardChangeOrigin::LocalRestore);
        assert_eq!(second, ClipboardChangeOrigin::LocalCapture);
    }
}
