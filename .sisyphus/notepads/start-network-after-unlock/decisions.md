# Decisions – start-network-after-unlock

## 2026-02-07 Lifecycle gating

- AppLifecycleCoordinator is the single choke point for watcher+network boot. It now performs an early `LifecycleState::Ready` check and exits without touching watcher/network, ensuring Ready remains the sole trigger for network start.
