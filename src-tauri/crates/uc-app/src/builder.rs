use std::sync::Arc;
use uc_core::ports::{AutostartPort, UiPort};

use crate::AppDeps;

/// Builder for assembling the application runtime.
pub struct AppBuilder {
    autostart: Option<Arc<dyn AutostartPort>>,
    ui_port: Option<Arc<dyn UiPort>>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            autostart: None,
            ui_port: None,
        }
    }

    pub fn with_autostart(mut self, autostart: Arc<dyn AutostartPort>) -> Self {
        self.autostart = Some(autostart);
        self
    }

    pub fn with_ui_port(mut self, ui_port: Arc<dyn UiPort>) -> Self {
        self.ui_port = Some(ui_port);
        self
    }

    pub fn build(self) -> anyhow::Result<App> {
        Ok(App {
            // Builder doesn't use AppDeps - it's None for backward compatibility
            deps: None,
            autostart: self.autostart.ok_or_else(|| {
                anyhow::anyhow!("AutostartPort is required")
            })?,
            ui_port: self.ui_port.ok_or_else(|| {
                anyhow::anyhow!("UiPort is required")
            })?,
        })
    }
}

/// The application runtime.
pub struct App {
    /// Dependency grouping for direct construction (preferred for new code)
    /// 用于直接构造的依赖分组（新代码推荐）
    pub deps: Option<AppDeps>,

    /// Public fields for backward compatibility with AppBuilder
    /// 用于与 AppBuilder 向后兼容的公共字段
    pub autostart: Arc<dyn AutostartPort>,
    pub ui_port: Arc<dyn UiPort>,
}

impl App {
    /// Create new App instance from dependencies
    /// 从依赖创建新的 App 实例
    ///
    /// This constructor signature IS the dependency manifest.
    /// 这个构造函数签名就是依赖清单。
    ///
    /// All dependencies must be provided - no defaults, no optionals.
    /// 必须提供所有依赖 - 无默认值，无可选字段.
    pub fn new(deps: AppDeps) -> Self {
        // Extract the ports needed for backward compatibility fields
        // Extract both ports at once to avoid partial move issues
        let (autostart, ui_port) = (deps.autostart.clone(), deps.ui_port.clone());

        Self {
            // Store deps internally for use case creation
            // 在内部存储 deps 用于 use case 创建
            deps: Some(deps),
            // Populate backward compatibility fields from deps
            // 从 deps 填充向后兼容字段
            autostart,
            ui_port,
        }
    }
}
