# Space Access (New Space) Draft Notes

## 2026-02-07

- Captured `SetupOrchestrator` regression and fix plan in `docs/plans/2026-02-07-setup-orchestrator-shared-state.md`.
- Key idea: instantiate `SetupOrchestrator` once inside `AppRuntime`, expose `runtime.setup_orchestrator()`, and wire Tauri commands to reuse the shared arc so new-space flows stop hitting `Welcome → SubmitPassphrase` invalid transitions.
- Regression guard: new `uc-tauri` test exercises `StartNewSpace` + `SubmitPassphrase` on different clones to ensure state persistence.
- Implementation complete: `AppRuntime` stores an `Arc<SetupOrchestrator>` (built with the shared lifecycle coordinator), commands now call `runtime.setup_orchestrator()`, and the regression test asserts `Arc::ptr_eq` plus shared state observation.
- Space-access follow-up plan: created Manus files (`task_plan.md`, `findings.md`, `progress.md`) to manage Task 1 scope—focus on rerouting `CreateEncryptedSpace` through `SpaceAccessOrchestrator`, migrating CompleteOnboarding semantics to `MarkSetupComplete`, and expanding `SetupStateMachine` tables for regression safety.
- Outstanding questions: how `SpaceAccessOrchestrator` exposes success/failure events back to Setup（目前仅同步 dispatch，不存在回调），以及 `Timer`/`SessionId` 依赖如何注入 Setup usecase。
- Risks logged: `SpaceAccess` actions beyond Sponsor init remain `ActionNotImplemented`; wiring too early may block Join flow tasks, so plan enforces “New Space only” guardrails.
