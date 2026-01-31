use super::*;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn framing_round_trips_single_frame() {
    let (mut client, mut server) = tokio::io::duplex(64 * 1024);

    let payload = br#"{\"k\":\"v\"}"#.to_vec();
    let expected = payload.clone();
    let write_task =
        tokio::spawn(async move { write_length_prefixed(&mut client, &payload).await });

    let read = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES).await;
    let wrote = write_task.await.expect("write task");

    wrote.expect("write ok");
    assert_eq!(read.expect("read ok").expect("some"), expected);
}

#[tokio::test]
async fn framing_rejects_oversize_frame() {
    let (mut client, mut server) = tokio::io::duplex(64 * 1024);

    let oversize = vec![0u8; MAX_PAIRING_FRAME_BYTES + 1];
    tokio::spawn(async move {
        let len = (oversize.len() as u32).to_be_bytes();
        let _ = client.write_all(&len).await;
        let _ = client.write_all(&oversize).await;
    });

    let err = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES)
        .await
        .expect_err("should error");
    assert!(err.to_string().contains("exceeds max"));
}

#[tokio::test]
async fn framing_eof_at_boundary_is_clean() {
    let (mut client, mut server) = tokio::io::duplex(64 * 1024);

    // Close client immediately without writing anything
    drop(client);

    let result = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES).await;
    assert!(result.expect("read ok").is_none());
}

#[tokio::test]
async fn framing_eof_mid_prefix_is_error() {
    let (mut client, mut server) = tokio::io::duplex(64 * 1024);

    tokio::spawn(async move {
        // Write 2 bytes (half of length prefix) then close
        let _ = client.write_all(&[0, 0]).await;
    });

    let err = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES)
        .await
        .expect_err("should error");

    // Should be UnexpectedEof
    let root = err.root_cause();
    let msg = root.to_string().to_lowercase();
    assert!(
        msg.contains("unexpected eof")
            || msg.contains("early eof")
            || msg.contains("failed to fill whole buffer"),
        "expected unexpected eof, got: {}",
        msg
    );
}

#[tokio::test]
async fn framing_eof_mid_payload_is_error() {
    let (mut client, mut server) = tokio::io::duplex(64 * 1024);

    tokio::spawn(async move {
        let len = 10u32.to_be_bytes();
        let _ = client.write_all(&len).await;
        // Write 5 bytes of payload (expecting 10)
        let _ = client.write_all(&[1, 2, 3, 4, 5]).await;
    });

    let err = read_length_prefixed(&mut server, MAX_PAIRING_FRAME_BYTES)
        .await
        .expect_err("should error");

    let root = err.root_cause();
    let msg = root.to_string().to_lowercase();
    assert!(
        msg.contains("unexpected eof")
            || msg.contains("early eof")
            || msg.contains("failed to fill whole buffer"),
        "expected unexpected eof, got: {}",
        msg
    );
}
