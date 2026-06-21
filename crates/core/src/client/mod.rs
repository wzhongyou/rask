use async_trait::async_trait;
use crate::{Message, error::Result};

pub mod openai;
pub mod anthropic;

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn chat(&self, messages: &[Message]) -> Result<String>;
}

pub fn infer_provider(model: &str) -> &'static str {
    if model.starts_with("claude-") {
        "anthropic"
    } else if model.starts_with("glm-") {
        "glm"
    } else if model.starts_with("deepseek-") {
        "deepseek"
    } else {
        "openai"
    }
}
