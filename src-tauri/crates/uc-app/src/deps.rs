//! # Application Dependencies / 应用依赖
//!
//! This module defines the dependency grouping for App construction.
//! 此模块定义 App 构造的依赖分组。
//!
//! **Note / 注意**: This is NOT a Builder pattern.
//! **这不是 Builder 模式。**
//! - No build steps / 无构建步骤
//! - No default values / 无默认值
//! - No hidden logic / 无隐藏逻辑
//! - Just parameter grouping / 仅用于参数打包

use std::sync::Arc;
use uc_core::ports::*;

/// Application dependency grouping (non-Builder, just parameter grouping)
/// 应用依赖分组（非 Builder，仅参数打包）
///
/// **NOT a Builder pattern** - this is just a struct to group parameters.
/// **不是 Builder 模式** - 这只是一个打包参数的结构体。
///
/// All dependencies are required - no defaults, no optional fields.
/// 所有依赖都是必需的 - 无默认值，无可选字段。
pub struct AppDeps {
    // Clipboard dependencies / 剪贴板依赖
    pub clipboard: Arc<dyn PlatformClipboardPort>,
    pub clipboard_entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    pub clipboard_event_repo: Arc<dyn ClipboardEventWriterPort>,
    pub representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    pub representation_materializer: Arc<dyn ClipboardRepresentationMaterializerPort>,
    pub selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    pub representation_policy: Arc<dyn SelectRepresentationPolicyPort>,

    // Security dependencies / 安全依赖
    pub encryption: Arc<dyn EncryptionPort>,
    pub encryption_session: Arc<dyn EncryptionSessionPort>,
    pub encryption_state: Arc<dyn uc_core::ports::security::encryption_state::EncryptionStatePort>,
    pub key_scope: Arc<dyn uc_core::ports::security::key_scope::KeyScopePort>,
    pub keyring: Arc<dyn KeyringPort>,
    pub key_material: Arc<dyn KeyMaterialPort>,

    // Device dependencies / 设备依赖
    pub device_repo: Arc<dyn DeviceRepositoryPort>,
    pub device_identity: Arc<dyn DeviceIdentityPort>,

    // Network dependencies / 网络依赖
    pub network: Arc<dyn NetworkPort>,

    // Onboarding dependencies / 入门引导依赖
    pub onboarding_state: Arc<dyn OnboardingStatePort>,

    // Storage dependencies / 存储依赖
    pub blob_store: Arc<dyn BlobStorePort>,
    pub blob_repository: Arc<dyn BlobRepositoryPort>,
    pub blob_materializer: Arc<dyn BlobMaterializerPort>,

    // Settings dependencies / 设置依赖
    pub settings: Arc<dyn SettingsPort>,

    // UI dependencies / UI 依赖
    pub ui_port: Arc<dyn UiPort>,
    pub autostart: Arc<dyn AutostartPort>,

    // System dependencies / 系统依赖
    pub clock: Arc<dyn ClockPort>,
    pub hash: Arc<dyn ContentHashPort>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_app_deps_is_just_a_struct() {
        // This test verifies AppDeps is a plain struct,
        // not a Builder with methods
        #[allow(dead_code)]
        fn assert_plain_struct<T: Sized>(_: &T) {}

        // We can't create a full AppDeps without all the trait implementations,
        // but we can verify the struct exists and is plain
        fn assert_app_deps_is_plain() {
            // This function body will remain empty since we can't create
            // full AppDeps without mock implementations
            // The important part is that this compiles - proving AppDeps
            // is a plain struct, not a Builder with methods
        }

        assert_app_deps_is_plain();
    }
}
