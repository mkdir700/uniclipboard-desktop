/// Infra 层的 Mapper
///
/// 语义：
/// 将一个 Domain 对象映射为某种存储行（Row / Record / DTO）
///
/// 约束：
/// - 只允许 Domain -> Infra
/// - 不允许反向依赖
pub trait Mapper<D, R>: Sync + Send {
    fn to_row(&self, domain: &D) -> R;
}
