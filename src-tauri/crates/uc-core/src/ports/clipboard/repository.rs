use anyhow::Result;
use async_trait::async_trait;

use crate::clipboard::{ClipboardContent, ClipboardContentView};

/// ClipboardRepository
///
/// 负责剪切板快照（ClipboardContent）的持久化与查询。
///
/// 设计约定：
/// - ClipboardContent 是聚合根，代表一次完整的复制/剪切事件
/// - items 仅作为 snapshot 的组成部分存在
/// - 所有写入操作必须是原子的（snapshot + items）
#[async_trait]
pub trait ClipboardRepositoryPort: Send + Sync {
    /// 保存一条剪切板快照
    ///
    /// 语义：
    /// - 表示“一次复制事件”
    /// - 内部应以事务方式写入
    /// - 若 content_hash 已存在，可选择忽略或直接返回成功（幂等）
    async fn save(&self, content: ClipboardContent) -> Result<()>;

    /// 根据 content_hash 判断是否已存在该剪切板快照
    ///
    /// 用于：
    /// - watcher 去重
    /// - 防止远端同步回环
    async fn exists(&self, content_hash: &str) -> Result<bool>;

    /// 获取最近的剪切板快照列表(按时间倒序)
    ///
    /// 约定：
    /// - 返回值按时间倒序
    async fn list_recent_views(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardContentView>>;

    /// 根据 content_hash 获取完整剪切板快照
    ///
    /// 用于：
    /// - UI 选中后回放
    /// - 网络同步后的本地重建
    async fn get_by_hash(&self, content_hash: &str) -> Result<Option<ClipboardContent>>;

    /// 软删除一条剪切板快照
    ///
    /// 语义：
    /// - 不会物理删除数据
    /// - snapshot 在默认列表中不可见
    async fn soft_delete(&self, content_hash: &str) -> Result<()>;
}
