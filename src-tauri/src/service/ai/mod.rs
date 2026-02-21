use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

pub mod openai;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn stream(&self, prompt: &str) -> Result<BoxStream<'static, Result<String>>>;
}
