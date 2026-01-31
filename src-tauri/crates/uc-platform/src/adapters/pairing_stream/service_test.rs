use super::framing::{write_length_prefixed, MAX_PAIRING_FRAME_BYTES};
use super::service::{PairingStreamConfig, PairingStreamService};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use uc_core::network::{NetworkEvent, PairingMessage, PairingRequest};

#[tokio::test]
async fn inbound_stream_emits_pairing_message() {
    let (event_tx, mut event_rx) = mpsc::channel(1);
    let service = PairingStreamService::for_tests(event_tx, PairingStreamConfig::default());
    let (mut client, server) = tokio::io::duplex(64 * 1024);

    let handle: tokio::task::JoinHandle<anyhow::Result<()>> =
        service.handle_incoming_stream("peer-1".to_string(), server);
    let message = PairingMessage::Request(PairingRequest {
        session_id: "session-1".to_string(),
        device_name: "device-a".to_string(),
        device_id: "device-a".to_string(),
        peer_id: "peer-a".to_string(),
        identity_pubkey: vec![1; 32],
        nonce: vec![2; 16],
    });
    let payload = serde_json::to_vec(&message).expect("serialize message");
    let write_task =
        tokio::spawn(async move { write_length_prefixed(&mut client, &payload).await });

    let event = timeout(Duration::from_secs(2), event_rx.recv())
        .await
        .expect("event timeout")
        .expect("event");
    assert!(matches!(
        event,
        NetworkEvent::PairingMessageReceived { peer_id, message }
            if peer_id == "peer-1" && matches!(message, PairingMessage::Request(_))
    ));

    let write_result = write_task.await.expect("write task");
    write_result.expect("write ok");
    service
        .close_pairing_session("session-1".to_string(), None)
        .await
        .expect("close session");

    let result = handle.await.expect("pairing stream task");
    result.expect("pairing stream ok");
}

#[tokio::test]
async fn oversize_frame_closes_session() {
    let (event_tx, mut event_rx) = mpsc::channel(1);
    let service = PairingStreamService::for_tests(event_tx, PairingStreamConfig::default());
    let (mut client, server) = tokio::io::duplex(64 * 1024);

    let handle: tokio::task::JoinHandle<anyhow::Result<()>> =
        service.handle_incoming_stream("peer-2".to_string(), server);
    let oversize = vec![0u8; MAX_PAIRING_FRAME_BYTES + 1];
    let write_task = tokio::spawn(async move {
        let len = (oversize.len() as u32).to_be_bytes();
        client.write_all(&len).await.expect("write len");
        client.write_all(&oversize).await.expect("write payload");
        client.shutdown().await.expect("shutdown");
    });

    let result = handle.await.expect("pairing stream task");
    assert!(result.is_err());
    assert!(event_rx.try_recv().is_err());

    write_task.await.expect("write task");
}

#[tokio::test]
async fn early_eof_does_not_panic_session_task() {
    let (event_tx, mut event_rx) = mpsc::channel(1);
    let service = PairingStreamService::for_tests(event_tx, PairingStreamConfig::default());
    let (mut client, server) = tokio::io::duplex(64 * 1024);

    let handle: tokio::task::JoinHandle<anyhow::Result<()>> =
        service.handle_incoming_stream("peer-4".to_string(), server);
    let message = PairingMessage::Request(PairingRequest {
        session_id: "session-4".to_string(),
        device_name: "device-b".to_string(),
        device_id: "device-b".to_string(),
        peer_id: "peer-b".to_string(),
        identity_pubkey: vec![3; 32],
        nonce: vec![4; 16],
    });
    let payload = serde_json::to_vec(&message).expect("serialize message");
    write_length_prefixed(&mut client, &payload)
        .await
        .expect("write payload");
    let event = timeout(Duration::from_secs(2), event_rx.recv())
        .await
        .expect("event timeout")
        .expect("event");
    assert!(matches!(
        event,
        NetworkEvent::PairingMessageReceived { peer_id, message }
            if peer_id == "peer-4" && matches!(message, PairingMessage::Request(_))
    ));
    client.shutdown().await.expect("shutdown");

    let _result = handle.await.expect("pairing stream task");
}

#[tokio::test]
async fn idle_timeout_closes_session() {
    let (event_tx, mut event_rx) = mpsc::channel(1);
    let config = PairingStreamConfig {
        idle_timeout: Duration::from_millis(50),
        ..PairingStreamConfig::default()
    };
    let service = PairingStreamService::for_tests(event_tx, config);
    let (_client, server) = tokio::io::duplex(64 * 1024);

    let handle: tokio::task::JoinHandle<anyhow::Result<()>> =
        service.handle_incoming_stream("peer-3".to_string(), server);
    let result = timeout(Duration::from_secs(1), handle)
        .await
        .expect("timeout")
        .expect("pairing stream task");
    assert!(result.is_err());
    assert!(event_rx.try_recv().is_err());
}
