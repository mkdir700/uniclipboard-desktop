//! Length-delimited framing helpers for pairing streams.

use anyhow::{anyhow, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Read a single length-prefixed frame enforcing an upper bound.
pub async fn read_length_prefixed<R>(reader: &mut R, max_frame_bytes: usize) -> Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > max_frame_bytes {
        return Err(anyhow!("frame exceeds max: {} > {}", len, max_frame_bytes));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

#[cfg(test)]
mod framing_test;
