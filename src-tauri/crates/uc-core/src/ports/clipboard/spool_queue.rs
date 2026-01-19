use crate::ids::RepresentationId;

#[derive(Debug, Clone)]
pub struct SpoolRequest {
    pub rep_id: RepresentationId,
    pub bytes: Vec<u8>,
}

#[async_trait::async_trait]
pub trait SpoolQueuePort: Send + Sync {
    async fn enqueue(&self, request: SpoolRequest) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::SpoolRequest;
    use crate::ids::RepresentationId;

    #[test]
    fn spool_request_is_clone() {
        let req = SpoolRequest {
            rep_id: RepresentationId::new(),
            bytes: vec![1, 2, 3],
        };
        let _clone = req.clone();
    }
}
