//! Mock implementations of clipboard ports for testing.
//!
//! This module provides mock implementations using `mockall` for unit testing
//! clipboard-related functionality without requiring real infrastructure.

use mockall::mock;
use crate::clipboard::{ClipboardContent, ClipboardContentView, ClipboardDecisionSnapshot};
use crate::ports::{ClipboardRepositoryPort, LocalClipboardPort, ClipboardHistoryPort};
use async_trait::async_trait;

/// Mock implementation of [`ClipboardRepositoryPort`].
///
/// Use this for testing code that depends on clipboard persistence
/// without requiring a real database.
mock! {
    pub Repo {}

    #[async_trait]
    impl ClipboardRepositoryPort for Repo {
        async fn save(&self, content: ClipboardContent) -> anyhow::Result<()>;
        async fn exists(&self, content_hash: &str) -> anyhow::Result<bool>;
        async fn list_recent_views(
            &self,
            limit: usize,
            offset: usize,
        ) -> anyhow::Result<Vec<ClipboardContentView>>;
        async fn get_by_hash(&self, content_hash: &str) -> anyhow::Result<Option<ClipboardContent>>;
        async fn soft_delete(&self, content_hash: &str) -> anyhow::Result<()>;
    }
}

/// Mock implementation of [`LocalClipboardPort`].
///
/// Use this for testing code that depends on system clipboard access
/// without requiring a real clipboard backend.
mock! {
    pub Clipboard {}

    #[async_trait]
    impl LocalClipboardPort for Clipboard {
        async fn read(&self) -> anyhow::Result<ClipboardContent>;
        async fn write(&self, content: ClipboardContent) -> anyhow::Result<()>;
    }
}

/// Mock implementation of [`ClipboardHistoryPort`].
///
/// Use this for testing code that queries clipboard decision history
/// without requiring a real database.
mock! {
    pub History {}

    #[async_trait]
    impl ClipboardHistoryPort for History {
        async fn get_snapshot_decision(&self, hash: &str)
            -> anyhow::Result<Option<ClipboardDecisionSnapshot>>;
    }
}
