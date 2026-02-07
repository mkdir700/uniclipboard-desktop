# Decisions

## 2026-02-07

- Rewrote to remove references to non-existent / and send serialized payloads through .
- Exported from and enabled module registration via .

## 2026-02-07

- Rewrote space_access/network_adapter.rs to remove references to missing ProtocolMessage and SpaceAccessCodec, and send serialized payloads through send_pairing_on_session.
- Exported SpaceAccessNetworkAdapter from space_access/mod.rs and registered module via mod network_adapter;.
