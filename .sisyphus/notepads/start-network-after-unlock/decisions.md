# Decisions – start-network-after-unlock

## 2026-02-07 Lifecycle gating

- AppLifecycleCoordinator is the single choke point for watcher+network boot. It now performs an early `LifecycleState::Ready` check and exits without touching watcher/network, ensuring Ready remains the sole trigger for network start.

## 2026-02-08 Adapter start semantics

- `NetworkControlPort::start_network()` in `Libp2pNetworkAdapter` now owns an internal CAS-driven lifecycle (`Idle -> Starting -> Started`, with `Failed -> Idle` rollback) so callers get idempotent `Ok(())` on duplicate starts and can retry after transient startup failures.
