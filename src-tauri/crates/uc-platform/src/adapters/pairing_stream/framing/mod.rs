//! Length-delimited framing helpers for pairing streams.

use anyhow::{anyhow, Result};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{trace, warn};

/// Maximum frame size accepted on pairing streams (16 KiB).
pub const MAX_PAIRING_FRAME_BYTES: usize = 16 * 1024;

/// Write a length-prefixed payload to the provided async writer.
pub async fn write_length_prefixed<W>(writer: &mut W, payload: &[u8]) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let len: u32 = payload
        .len()
        .try_into()
        .map_err(|_| anyhow!("frame too large for u32: {} bytes", payload.len()))?;

    trace!(
        stage = "write_len_prefix",
        len = len,
        "writing frame length"
    );
    if let Err(e) = writer.write_all(&len.to_be_bytes()).await {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            warn!(
                stage = "write_len_prefix",
                error = %e,
                expected = 4,
                "unexpected eof writing length"
            );
        }
        return Err(e.into());
    }

    trace!(stage = "write_payload", len = len, "writing frame payload");
    if let Err(e) = writer.write_all(payload).await {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            warn!(
                stage = "write_payload",
                error = %e,
                expected = len,
                "unexpected eof writing payload"
            );
        }
        return Err(e.into());
    }

    writer.flush().await?;
    Ok(())
}

/// Read a single length-prefixed frame enforcing an upper bound.
pub async fn read_length_prefixed<R>(reader: &mut R, max_frame_bytes: usize) -> Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    trace!(stage = "read_len_prefix", "reading frame length");
    if let Err(e) = reader.read_exact(&mut len_buf).await {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            warn!(
                stage = "read_len_prefix",
                error = %e,
                expected = 4,
                "unexpected eof reading length"
            );
        }
        return Err(e.into());
    }

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > max_frame_bytes {
        return Err(anyhow!("frame exceeds max: {} > {}", len, max_frame_bytes));
    }

    let mut buf = vec![0u8; len];
    trace!(stage = "read_payload", len = len, "reading frame payload");
    if let Err(e) = reader.read_exact(&mut buf).await {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            warn!(
                stage = "read_payload",
                error = %e,
                expected = len,
                "unexpected eof reading payload"
            );
        }
        return Err(e.into());
    }
    Ok(buf)
}

#[cfg(test)]
mod framing_test;
