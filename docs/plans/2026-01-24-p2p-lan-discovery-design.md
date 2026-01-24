P2P LAN Discovery (mDNS) Design

This document defines the new-architecture design for LAN-only peer discovery and
reachability observation in UniClipboard. It intentionally excludes pairing and
data transfer; those are future steps.

## Goals / 目标

- Implement LAN discovery via mDNS in the new architecture (`uc-platform`).
- Surface discovery and reachability observations through `NetworkEvent`.
- Keep `NetworkPort` as minimal, stable network capabilities (no business semantics).
- Ensure identity is stable across restarts (already handled by identity store).

## Non-Goals / 非目标

- No DHT / relay / hole punching.
- No pairing handshake or persistence.
- No clipboard message transfer.
- No explicit connection lifecycle APIs in `NetworkPort`.

## Architecture / 架构

We implement a concrete `Libp2pNetworkAdapter` inside `uc-platform` that conforms
to `NetworkPort` without exposing transport details. The adapter owns a libp2p
Swarm and runs an internal async task to process events.

Key components:

1. **Libp2pNetworkAdapter**
   - Implements `NetworkPort` using libp2p + mDNS behaviour.
   - Holds caches of discovered peers and reachable peers.
   - Uses stored identity to derive a stable `PeerId`.

2. **SwarmTask**
   - Runs the Swarm event loop.
   - Translates observations into `NetworkEvent` for consumers.
   - Updates caches based on observed events.

3. **PeerCaches**
   - `discovered_peers`: peers seen via mDNS.
   - `reachable_peers`: best-effort reachability set (not a strict connection state).
   - The term “reachable” is used to avoid over-claiming connection semantics.

## Data Flow & Event Semantics / 数据流与事件语义

- mDNS discovery emits `PeerDiscovered` and `PeerLost` events.
- Reachability updates are derived from observed Swarm events (dial success/failure,
  connection close, etc.), but **events are observations, not state machine callbacks**.
- The adapter does **not** auto-dial on discovery. It only attempts a dial when
  an explicit action occurs (e.g. future `send`/`request` operations).
- `reachable_peers` is a best-effort view and may diverge from actual connectivity
  in relay/store-and-forward scenarios.

## Error Handling / 错误处理

- All network failures are observable (log + event), not silent.
- Adapter methods return explicit errors without leaking libp2p internals.
- Discovery failures should not crash the app; they degrade to “no peers found”.

## Testing / 测试

- Unit tests for cache update logic and event mapping.
- Integration tests (later step) to validate cross-device discovery in LAN.

## Rollback / 回滚

- Revert adapter wiring to `PlaceholderNetworkPort`.
- Remove Swarm task startup; leave identity store intact.
