use crate::setup::SetupState;

#[async_trait::async_trait]
pub trait SetupEventPort: Send + Sync {
    async fn emit_setup_state_changed(&self, state: SetupState, session_id: Option<String>);
}
