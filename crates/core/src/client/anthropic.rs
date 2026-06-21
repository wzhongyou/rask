use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Message, error::{RaskError, Result}};
use super::AiClient;

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self { client: Client::new(), api_key: api_key.into(), model: model.into() }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: &'a [Message],
}

#[derive(Deserialize)]
struct ChatResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[async_trait]
impl AiClient for AnthropicClient {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        let resp = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&ChatRequest { model: &self.model, max_tokens: 4096, messages })
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(RaskError::Api { status, message });
        }

        let body: ChatResponse = resp.json().await?;
        Ok(body.content.into_iter().next().map(|b| b.text).unwrap_or_default())
    }
}
