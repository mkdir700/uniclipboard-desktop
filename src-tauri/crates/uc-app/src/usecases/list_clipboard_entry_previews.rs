// //! 前端列表
// //! 最近 N 条复制记录
// //! 启动时加载历史

// use crate::{models::ClipboardEntryPreview, ports::ClipboardEntryPreviewQueryPort};
// use anyhow::Result;
// use uc_core::ports::ClipboardEntryRepositoryPort;

// pub struct ListClipboardEntryPreviewsUseCase<Q, S>
// where
//     Q: ClipboardEntryPreviewQueryPort,
//     S: ClipboardEntryRepositoryPort,
// {
//     entry_repo: Q,
//     selection_repo: S,
// }

// impl<Q, S> ListClipboardEntryPreviewsUseCase<Q, S>
// where
//     Q: ClipboardEntryPreviewQueryPort,
//     S: ClipboardEntryRepositoryPort,
// {
//     pub async fn execute(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntryPreview>> {
//         let previews = self.entry_repo.list_previews(limit, offset).await?;

//         Ok(previews)
//     }
// }
