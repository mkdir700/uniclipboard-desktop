# P2P Discovery Device Name Design

## Goal

Ensure the discovery list shows device names during the mDNS discovery phase.

## Root Cause

Discovery in the new libp2p adapter only stores `peer_id` and addresses. No device name is
collected until pairing messages arrive, so `device_name` remains `None` and the UI always
falls back to "unknown".

## Approach (Chosen)

Use libp2p Identify to read `agent_version` and parse device name in discovery phase. This is
the minimal change that preserves Hexagonal boundaries and does not require new protocols.

## Data Flow

1. libp2p swarm receives Identify info for a peer.
2. Parse `agent_version` in the strict format:
   `uniclipboard/<version>/<device_id>/<device_name>`
3. Update `PeerCaches.discovered_peers[peer_id].device_name`.
4. `get_p2p_peers` continues to read from `list_discovered_peers` without interface changes.

## Implementation Notes

- Add Identify behaviour to `Libp2pBehaviour`.
- Set local Identify `agent_version` to the strict format above using settings device name and
  the local device id.
- Parsing is best-effort; failures are debug-logged and do not fail discovery.
- No backward compatibility paths are included.

## Error Handling

- If `agent_version` does not match the strict format, skip update and log a debug line.
- If the peer is not yet in cache, log a debug line (event-driven visibility).

## Testing

- Unit test the parser with a valid `agent_version` and a malformed one.
- Verify that `PeerCaches` updates `device_name` for valid Identify info.
