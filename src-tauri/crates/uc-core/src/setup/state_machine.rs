//! Setup state machine.
//!
//! Defines a pure state transition function for the onboarding setup flow.

/// Setup flow state.
///
/// 设置流程状态。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupState {
    /// Welcome screen.
    ///
    /// 欢迎页。
    Welcome,
    /// Create-space passphrase input.
    ///
    /// 创建空间口令输入页。
    CreateSpacePassphrase { error: Option<String> },
}

/// Events that drive the setup flow.
///
/// 驱动设置流程的事件。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupEvent {
    /// User chooses to create a new space.
    ///
    /// 用户选择创建新空间。
    ChooseCreateSpace,
}

/// Side-effects produced by state transitions.
///
/// 状态迁移产生的副作用。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SetupAction {}

/// Pure setup state machine.
///
/// 纯状态机：不包含副作用。
pub struct SetupStateMachine;

impl SetupStateMachine {
    pub fn transition(state: SetupState, event: SetupEvent) -> (SetupState, Vec<SetupAction>) {
        match (state, event) {
            (SetupState::Welcome, SetupEvent::ChooseCreateSpace) => (
                SetupState::CreateSpacePassphrase { error: None },
                Vec::new(),
            ),
            (state, _event) => (state, Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SetupEvent, SetupState, SetupStateMachine};

    #[test]
    fn setup_state_machine_welcome_choose_create_transitions_to_create_passphrase() {
        let state = SetupState::Welcome;
        let (next, actions) = SetupStateMachine::transition(state, SetupEvent::ChooseCreateSpace);
        assert_eq!(next, SetupState::CreateSpacePassphrase { error: None });
        assert!(actions.is_empty());
    }
}
