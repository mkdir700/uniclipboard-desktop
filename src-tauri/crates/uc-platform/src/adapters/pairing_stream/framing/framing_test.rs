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
    assert_eq!(read.expect("read ok"), expected);
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
