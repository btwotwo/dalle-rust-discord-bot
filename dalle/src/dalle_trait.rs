use crate::DalleResponse;
use async_trait::async_trait;

#[async_trait]
pub trait DalleGenerator {
    async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>>;
    async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Vec<DalleResponse>>>;
}
