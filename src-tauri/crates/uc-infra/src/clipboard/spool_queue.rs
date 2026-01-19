use tokio::sync::mpsc;
use uc_core::ports::clipboard::{SpoolQueuePort, SpoolRequest};

pub struct MpscSpoolQueue {
    sender: mpsc::Sender<SpoolRequest>,
}

impl MpscSpoolQueue {
    pub fn new(sender: mpsc::Sender<SpoolRequest>) -> Self {
        Self { sender }
    }
}

#[async_trait::async_trait]
impl SpoolQueuePort for MpscSpoolQueue {
    async fn enqueue(&self, request: SpoolRequest) -> anyhow::Result<()> {
        self.sender
            .send(request)
            .await
            .map_err(|err| anyhow::anyhow!("spool queue closed: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::MpscSpoolQueue;
    use tokio::sync::mpsc;
    use uc_core::ids::RepresentationId;
    use uc_core::ports::clipboard::{SpoolQueuePort, SpoolRequest};

    #[tokio::test]
    async fn enqueues_request() {
        let (tx, mut rx) = mpsc::channel(1);
        let queue = MpscSpoolQueue::new(tx);
        let req = SpoolRequest {
            rep_id: RepresentationId::new(),
            bytes: vec![1, 2, 3],
        };

        queue.enqueue(req.clone()).await.expect("enqueue");
        let received = rx.recv().await.expect("recv");
        assert_eq!(received.rep_id, req.rep_id);
        assert_eq!(received.bytes, req.bytes);
    }
}
