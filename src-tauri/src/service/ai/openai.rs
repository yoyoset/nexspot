use super::LLMProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Option<Delta>,
    _message: Option<Delta>, // Non-streaming
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

pub struct OpenAIProvider {
    api_url: String,
    api_key: String,
    model: String,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        Self {
            api_url,
            api_key,
            model,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate(&self, _prompt: &str) -> Result<String> {
        // Not implemented (using stream primarily)
        Err(anyhow!("Use stream for now"))
    }

    async fn stream(&self, prompt: &str) -> Result<BoxStream<'static, Result<String>>> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("API Error: {}", response.status()));
        }

        let stream = response.bytes_stream().map(|chunk| {
            match chunk {
                Ok(bytes) => {
                    let s = String::from_utf8_lossy(&bytes);
                    // Parse SSE format (data: {...})
                    let mut result = String::new();
                    for line in s.lines() {
                        if line.starts_with("data: ") {
                            let data = line.trim_start_matches("data: ");
                            if data == "[DONE]" {
                                break;
                            }
                            // log::trace!("OpenAI SSE Chunk: {}", data);
                            if let Ok(json) = serde_json::from_str::<OpenAIResponse>(data) {
                                if let Some(choice) = json.choices.first() {
                                    if let Some(delta) = &choice.delta {
                                        if let Some(content) = &delta.content {
                                            result.push_str(content);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                Err(e) => Err(anyhow::Error::new(e)),
            }
        });

        Ok(Box::pin(stream))
    }
}
