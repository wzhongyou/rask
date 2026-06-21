use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Message, error::{RaskError, Result}};
use super::AiClient;

pub struct OpenAiClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiClient {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[async_trait]
impl AiClient for OpenAiClient {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&ChatRequest { model: &self.model, messages })
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(RaskError::Api { status, message });
        }

        let body: ChatResponse = resp.json().await?;
        Ok(body.choices.into_iter().next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }
}
