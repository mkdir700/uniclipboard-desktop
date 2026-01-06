//! Tests for [`CopyFromHistoryToSystemClipboard`] use case.

use std::sync::Arc;
use uc_app::use_cases::clipboard::copy_from_history_to_system_clipbaord::CopyFromHistoryToSystemClipboard;
use uc_core::clipboard::*;
use uc_core::ports::{ClipboardRepositoryPort, LocalClipboardPort};
use std::collections::BTreeMap;
use async_trait::async_trait;

// Mock implementations for uc-app tests
struct MockRepo {
    get_by_hash_result: Option<ClipboardContent>,
    should_error: bool,
}

#[async_trait]
impl ClipboardRepositoryPort for MockRepo {
    async fn save(&self, _content: ClipboardContent) -> anyhow::Result<()> {
        Ok(())
    }
    async fn exists(&self, _content_hash: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn list_recent_views(
        &self,
        _limit: usize,
        _offset: usize,
    ) -> anyhow::Result<Vec<uc_core::clipboard::ClipboardContentView>> {
        Ok(vec![])
    }
    async fn get_by_hash(&self, _content_hash: &str) -> anyhow::Result<Option<ClipboardContent>> {
        if self.should_error {
            Err(anyhow::anyhow!("Database error"))
        } else {
            Ok(self.get_by_hash_result.clone())
        }
    }
    async fn soft_delete(&self, _content_hash: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct MockClipboard {
    should_error: bool,
}

#[async_trait]
impl LocalClipboardPort for MockClipboard {
    async fn read(&self) -> anyhow::Result<ClipboardContent> {
        Ok(ClipboardContent {
            v: 1,
            ts_ms: 0,
            items: vec![],
            meta: BTreeMap::new(),
        })
    }
    async fn write(&self, _content: ClipboardContent) -> anyhow::Result<()> {
        if self.should_error {
            Err(anyhow::anyhow!("Clipboard write failed"))
        } else {
            Ok(())
        }
    }
}

fn create_use_case(
    repo: Arc<MockRepo>,
    clipboard: Arc<MockClipboard>,
) -> CopyFromHistoryToSystemClipboard<MockRepo, MockClipboard> {
    CopyFromHistoryToSystemClipboard::new(repo, clipboard)
}

fn create_test_content() -> ClipboardContent {
    ClipboardContent {
        v: 1,
        ts_ms: 1000,
        items: vec![ClipboardItem {
            mime: MimeType::text_plain(),
            data: ClipboardData::Text { text: "test content".to_string() },
            meta: BTreeMap::new(),
        }],
        meta: BTreeMap::new(),
    }
}

#[tokio::test]
async fn test_copy_from_history_success() {
    let content = create_test_content();
    let repo = Arc::new(MockRepo {
        get_by_hash_result: Some(content.clone()),
        should_error: false,
    });
    let clipboard = Arc::new(MockClipboard {
        should_error: false,
    });
    let hash = content.content_hash();

    let use_case = create_use_case(repo, clipboard);
    let result = use_case.execute(&hash).await;

    assert!(result.is_ok(), "Success case should return Ok(())");
}

#[tokio::test]
async fn test_copy_from_history_not_found() {
    let repo = Arc::new(MockRepo {
        get_by_hash_result: None,
        should_error: false,
    });
    let clipboard = Arc::new(MockClipboard {
        should_error: false,
    });

    let use_case = create_use_case(repo, clipboard);
    let result = use_case.execute("nonexistent-hash").await;

    assert!(result.is_ok(), "Not found should silently succeed");
}

#[tokio::test]
async fn test_copy_from_history_repo_error() {
    let repo = Arc::new(MockRepo {
        get_by_hash_result: None,
        should_error: true,
    });
    let clipboard = Arc::new(MockClipboard {
        should_error: false,
    });

    let use_case = create_use_case(repo, clipboard);
    let result = use_case.execute("error-hash").await;

    assert!(result.is_err(), "Repo error should propagate");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Database error") || err_msg.contains("error"));
}

#[tokio::test]
async fn test_copy_from_history_clipboard_error() {
    let content = create_test_content();
    let repo = Arc::new(MockRepo {
        get_by_hash_result: Some(content.clone()),
        should_error: false,
    });
    let clipboard = Arc::new(MockClipboard {
        should_error: true,
    });
    let hash = content.content_hash();

    let use_case = create_use_case(repo, clipboard);
    let result = use_case.execute(&hash).await;

    assert!(result.is_err(), "Clipboard error should propagate");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Clipboard") || err_msg.contains("failed"));
}
