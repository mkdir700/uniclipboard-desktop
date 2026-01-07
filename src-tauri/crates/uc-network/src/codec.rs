use futures::prelude::*;
use libp2p::{request_response::Codec, StreamProtocol};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

const PROTOCOL_NAME: &str = "/uniclipboard/1.0.0";

/// Request-response codec for pairing protocol
#[derive(Debug, Clone, Default)]
pub struct UniClipboardCodec;

/// Pairing request wrapper for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub message: Vec<u8>,
}

/// Pairing response wrapper for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingResponse {
    pub message: Vec<u8>,
}

impl Codec for UniClipboardCodec {
    type Protocol = StreamProtocol;
    type Request = PairingRequest;
    type Response = PairingResponse;

    fn read_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> Pin<Box<dyn futures::Future<Output = std::io::Result<Self::Request>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        T: AsyncRead + Unpin + Send + 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let mut buf = Vec::new();
            let mut limited = io.take(1024 * 64); // 64KB limit
            limited.read_to_end(&mut buf).await?;
            serde_json::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn read_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> Pin<Box<dyn futures::Future<Output = std::io::Result<Self::Response>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        T: AsyncRead + Unpin + Send + 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let mut buf = Vec::new();
            let mut limited = io.take(1024 * 64); // 64KB limit
            limited.read_to_end(&mut buf).await?;
            serde_json::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn write_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
        req: Self::Request,
    ) -> Pin<Box<dyn futures::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        T: AsyncWrite + Unpin + Send + 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let bytes = serde_json::to_vec(&req)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&bytes).await?;
            io.close().await?;
            Ok(())
        })
    }

    fn write_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
        res: Self::Response,
    ) -> Pin<Box<dyn futures::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        T: AsyncWrite + Unpin + Send + 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let bytes = serde_json::to_vec(&res)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            io.write_all(&bytes).await?;
            io.close().await?;
            Ok(())
        })
    }
}
