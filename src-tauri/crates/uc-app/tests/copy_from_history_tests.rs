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
    /// No-op mock save that ignores the provided clipboard content and always succeeds.
    ///
    /// This implementation is intended for tests: it does not persist or validate the `ClipboardContent` and simply returns success.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn example_save_noop() {
    ///     let repo = MockRepo { get_by_hash_result: None, should_error: false };
    ///     repo.save(create_test_content()).await.unwrap();
    /// }
    /// ```
    ///
    /// Returns `Ok(())` on success.
    async fn save(&self, _content: ClipboardContent) -> anyhow::Result<()> {
        Ok(())
    }
    /// Checks whether a clipboard content with the given hash exists in the repository.
    ///
    /// In this mock implementation used for tests, the function always returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn mock_repo_exists_always_false() {
    ///     let repo = MockRepo { get_by_hash_result: None, should_error: false };
    ///     let found = repo.exists("any-hash").await.unwrap();
    ///     assert!(!found);
    /// }
    /// ```
    async fn exists(&self, _content_hash: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
    /// Returns an empty list of recent clipboard content views, ignoring the provided `limit` and `offset`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a repository `repo` that implements `list_recent_views`,
    /// // the mock implementation always yields an empty vector:
    /// // let views = repo.list_recent_views(10, 0).await;
    /// // assert!(views.is_empty());
    /// ```
    async fn list_recent_views(
        &self,
        _limit: usize,
        _offset: usize,
    ) -> anyhow::Result<Vec<uc_core::clipboard::ClipboardContentView>> {
        Ok(vec![])
    }
    /// Return the mock repository's configured clipboard content for the given hash or a simulated error.
    ///
    /// This mock method ignores the provided `content_hash` and either returns the preconfigured
    /// `get_by_hash_result` or an error when `should_error` is set to `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::Arc;
    /// # use tokio::runtime::Runtime;
    /// // Construct a mock repo that will return a content.
    /// let repo = MockRepo { get_by_hash_result: Some(create_test_content()), should_error: false };
    /// let rt = Runtime::new().unwrap();
    /// let result = rt.block_on(async { repo.get_by_hash("any-hash").await }).unwrap();
    /// assert!(result.is_some());
    /// ```
    async fn get_by_hash(&self, _content_hash: &str) -> anyhow::Result<Option<ClipboardContent>> {
        if self.should_error {
            Err(anyhow::anyhow!("Database error"))
        } else {
            Ok(self.get_by_hash_result.clone())
        }
    }
    /// Simulates a successful soft-delete of a clipboard entry without modifying state.
    ///
    /// This mock implementation ignores the provided `content_hash` and always succeeds.
    ///
    /// # Returns
    ///
    /// `Ok(())` indicating the soft-delete was performed successfully (no-op).
    async fn soft_delete(&self, _content_hash: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct MockClipboard {
    should_error: bool,
}

#[async_trait]
impl LocalClipboardPort for MockClipboard {
    /// Provide a default ClipboardContent used by the mock clipboard.
    ///
    /// The returned content has version 1, timestamp 0, an empty items vector, and an empty metadata map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::tests::copy_from_history_tests::MockClipboard;
    /// # tokio_test::block_on(async {
    /// let clipboard = MockClipboard { should_error: false };
    /// let content = clipboard.read().await.unwrap();
    /// assert_eq!(content.v, 1);
    /// assert_eq!(content.items.len(), 0);
    /// assert!(content.meta.is_empty());
    /// # });
    /// ```
    async fn read(&self) -> anyhow::Result<ClipboardContent> {
        Ok(ClipboardContent {
            v: 1,
            ts_ms: 0,
            items: vec![],
            meta: BTreeMap::new(),
        })
    }
    /// Simulates writing clipboard content and optionally fails when the mock is configured.
    ///
    /// When `should_error` is true the mock returns an error to simulate a clipboard write failure;
    /// otherwise it succeeds.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    ///
    /// let mock = MockClipboard { should_error: false };
    /// // `ClipboardContent` can be constructed as needed for tests; here we assume a default is available.
    /// let content = ClipboardContent::default();
    /// block_on(mock.write(content)).unwrap();
    /// ```
    async fn write(&self, _content: ClipboardContent) -> anyhow::Result<()> {
        if self.should_error {
            Err(anyhow::anyhow!("Clipboard write failed"))
        } else {
            Ok(())
        }
    }
}

/// Constructs a `CopyFromHistoryToSystemClipboard` use case wired to the given mock repository and clipboard.
///
/// # Examples
///
/// ```
/// let repo = std::sync::Arc::new(MockRepo { get_by_hash_result: None, should_error: false });
/// let clipboard = std::sync::Arc::new(MockClipboard { should_error: false });
/// let uc = create_use_case(repo, clipboard);
/// ```
— A `CopyFromHistoryToSystemClipboard` instance configured to use the supplied mock repository and clipboard.
fn create_use_case(
    repo: Arc<MockRepo>,
    clipboard: Arc<MockClipboard>,
) -> CopyFromHistoryToSystemClipboard<MockRepo, MockClipboard> {
    CopyFromHistoryToSystemClipboard::new(repo, clipboard)
}

/// Constructs a sample `ClipboardContent` containing a single plain-text item with the text "test content".
///
/// # Examples
///
/// ```
/// let content = create_test_content();
/// assert_eq!(content.v, 1);
/// assert_eq!(content.items.len(), 1);
/// assert_eq!(content.items[0].mime, MimeType::text_plain());
/// if let ClipboardData::Text { text } = &content.items[0].data {
///     assert_eq!(text, "test content");
/// } else {
///     panic!("expected text data");
/// }
/// ```
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

/// Verifies that copying a clipboard entry from history to the system clipboard succeeds when the repository returns the entry and the clipboard write does not fail.
///
/// # Examples
///
/// ```
/// # async fn run_test() {
/// let content = create_test_content();
/// let repo = Arc::new(MockRepo {
///     get_by_hash_result: Some(content.clone()),
///     should_error: false,
/// });
/// let clipboard = Arc::new(MockClipboard { should_error: false });
/// let hash = content.content_hash();
///
/// let use_case = create_use_case(repo, clipboard);
/// let result = use_case.execute(&hash).await;
///
/// assert!(result.is_ok());
/// # }
/// ```
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

/// Verifies that a repository error is propagated as a failure by the use case.
///
/// Constructs a repository mock that returns an error from `get_by_hash` and asserts the use case
/// returns an `Err` whose message contains "Database error" or "error".
///
/// # Examples
///
/// ```
/// # use std::sync::Arc;
/// # tokio_test::block_on(async {
/// let repo = Arc::new(MockRepo { get_by_hash_result: None, should_error: true });
/// let clipboard = Arc::new(MockClipboard { should_error: false });
/// let use_case = create_use_case(repo, clipboard);
/// let result = use_case.execute("error-hash").await;
/// assert!(result.is_err());
/// # });
/// ```
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

/// Verifies that a clipboard write failure is returned as an error by the use case.
///
/// Sets up a repository that returns a previously saved history entry and a clipboard mock
/// that fails on write; executing the use case with the entry's hash must produce an error
/// indicating a clipboard/failure condition.
///
/// # Examples
///
/// ```
/// // Arrange: repo returns content, clipboard is configured to fail
/// let content = create_test_content();
/// let repo = Arc::new(MockRepo { get_by_hash_result: Some(content.clone()), should_error: false });
/// let clipboard = Arc::new(MockClipboard { should_error: true });
/// let use_case = create_use_case(repo, clipboard);
/// let hash = content.content_hash();
///
/// // Act: execute use case
/// let result = use_case.execute(&hash).await;
///
/// // Assert: error is propagated and mentions clipboard/failure
/// assert!(result.is_err());
/// let err_msg = result.unwrap_err().to_string();
/// assert!(err_msg.contains("Clipboard") || err_msg.contains("failed"));
/// ```
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