//! Default key scope implementation
//! 默认密钥范围实现

use anyhow::Result;
use uc_core::ports::security::key_scope::KeyScopePort;
use uc_core::security::model::KeyScope;
use uc_core::ports::security::key_scope::ScopeError;

/// Default key scope implementation
pub struct DefaultKeyScope {
    scope: KeyScope,
}

impl DefaultKeyScope {
    pub fn new() -> Self {
        Self {
            scope: KeyScope {
                profile_id: "default".to_string(),
            },
        }
    }
}

impl Default for DefaultKeyScope {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl KeyScopePort for DefaultKeyScope {
    async fn current_scope(&self) -> Result<KeyScope, ScopeError> {
        Ok(self.scope.clone())
    }
}
