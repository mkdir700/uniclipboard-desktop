# Learnings

- 2026-02-07: SpaceAccessExecutor now needs SpaceAccessTransportPort/ProofPort refs when constructed so every orchestrator or use case wiring must provide them alongside network/timer/store dependencies.

- 2026-02-07: SetupOrchestrator now stores Pairing/SpaceAccess orchestrators plus DiscoveryPort and pairing_session_id so any future wiring/tests must provide those arcs (runtime currently feeds placeholder implementations).

- 2026-02-07: Pairing verification state now derives from PairingOrchestrator events; SetupOrchestrator listens for `PairingVerificationRequired` to enter `JoinSpaceConfirmPeer`, so tests must pump those events when exercising join flows.

- 2026-02-07: SpaceAccessContext tracks joiner_offer, joiner_passphrase, proof_artifact, and sponsor_peer_id; SpaceAccessExecutor.transport must be mutable so transport adapters can send offers/proofs/results.

- 2026-02-07: SetupOrchestrator::verify_passphrase dispatches SubmitPassphrase and capture_context normalizes VerifyPassphrase → Submit to avoid invalid JoinSpaceInputPassphrase transitions.

- 2026-02-07: Confirm peer trust flow now exposed end-to-end (SetupPage.tsx -> confirmPeerTrust API -> confirm_peer_trust command -> SetupOrchestrator.confirm_peer_trust), so future setup steps should call the API instead of canceling the flow.

- 2026-02-07: SetupOrchestrator unit tests cannot drive real pairing yet (StartJoinSpaceAccess still unimplemented), so tests must seed `selected_peer_id`/`pairing_session_id` and force context state transitions instead of calling `submit_passphrase`/`confirm_peer_trust` directly.

- 2026-02-07: SpaceAccessOrchestrator tests must inject joiner offers/passphrases and sponsor peer ids through the shared context before dispatching events, otherwise actions like key derivation/persistence panic due to missing context.

- 2026-02-07: AppRuntime 现在通过 SetupRuntimePorts 注入真实的 Pairing/SpaceAccess/Discovery 依赖，bootstrap wiring 必须在构造 runtime 前创建这些 orchestrator/adapters；单元测试若仍用 AppRuntime::new() 会得到占位实现。

- 2026-02-07: StartJoinSpaceAccess action 不再抛 ActionNotImplemented，而是以 LifecycleFailed 形式提示 join space access 流程尚未接好，前端可据此展示合理错误。
