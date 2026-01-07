#[async_trait::async_trait]
pub trait AppRuntimePort<E>: Send + Sync {
    async fn emit(&self, event: E);
    async fn exit(&self);
}
