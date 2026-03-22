#[async_trait::async_trait]
pub trait AgentBackend: Send + Sync {
    async fn query(&self, text: &str) -> anyhow::Result<String>;
}
