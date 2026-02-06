/// Setup status persisted across app restarts.
///
/// 设置流程持久化状态。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SetupStatus {
    pub has_completed: bool,
}

impl Default for SetupStatus {
    fn default() -> Self {
        Self {
            has_completed: false,
        }
    }
}
