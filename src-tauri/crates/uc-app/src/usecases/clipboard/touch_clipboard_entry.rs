use anyhow::Result;
use std::sync::Arc;

use uc_core::ids::EntryId;
use uc_core::ports::{ClipboardEntryRepositoryPort, ClockPort};

/// Update clipboard entry active time.
///
/// 更新剪贴板条目的活跃时间。
pub struct TouchClipboardEntryUseCase {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    clock: Arc<dyn ClockPort>,
}

impl TouchClipboardEntryUseCase {
    pub fn new(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        clock: Arc<dyn ClockPort>,
    ) -> Self {
        Self { entry_repo, clock }
    }

    pub async fn execute(&self, entry_id: &EntryId) -> Result<bool> {
        let now_ms = self.clock.now_ms();

        self.entry_repo.touch_entry(entry_id, now_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::TouchClipboardEntryUseCase;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use uc_core::clipboard::{ClipboardEntry, ClipboardSelectionDecision};
    use uc_core::ids::EntryId;
    use uc_core::ports::{ClipboardEntryRepositoryPort, ClockPort};

    struct MockEntryRepository {
        touched_at: Arc<Mutex<Option<i64>>>,
    }

    struct MockClock {
        now_ms: i64,
    }

    #[async_trait]
    impl ClipboardEntryRepositoryPort for MockEntryRepository {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &ClipboardSelectionDecision,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> anyhow::Result<Option<ClipboardEntry>> {
            Ok(None)
        }

        async fn list_entries(
            &self,
            _limit: usize,
            _offset: usize,
        ) -> anyhow::Result<Vec<ClipboardEntry>> {
            Ok(vec![])
        }

        async fn touch_entry(
            &self,
            _entry_id: &EntryId,
            active_time_ms: i64,
        ) -> anyhow::Result<bool> {
            if let Ok(mut touched_at) = self.touched_at.lock() {
                *touched_at = Some(active_time_ms);
            }
            Ok(true)
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> anyhow::Result<()> {
            Ok(())
        }
    }

    impl ClockPort for MockClock {
        fn now_ms(&self) -> i64 {
            self.now_ms
        }
    }

    #[tokio::test]
    async fn execute_uses_clock_now_ms_for_touch() {
        let touched_at = Arc::new(Mutex::new(None));
        let entry_repo = Arc::new(MockEntryRepository {
            touched_at: touched_at.clone(),
        });
        let clock = Arc::new(MockClock { now_ms: 1234 });
        let uc = TouchClipboardEntryUseCase::new(entry_repo, clock);
        let entry_id = EntryId::from("entry-1");

        let result = uc.execute(&entry_id).await.unwrap();

        assert!(result);
        assert_eq!(*touched_at.lock().unwrap(), Some(1234));
    }
}
