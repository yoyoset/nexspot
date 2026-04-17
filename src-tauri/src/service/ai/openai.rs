use super::LLMProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Complex(Vec<ContentPart>),
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: MessageContent,
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
    async fn generate(&self, _prompt: &str, _image: Option<&str>) -> Result<String> {
        // Not implemented (using stream primarily)
        Err(anyhow!("Use stream for now"))
    }

    async fn stream(
        &self,
        prompt: &str,
        image: Option<&str>,
    ) -> Result<BoxStream<'static, Result<String>>> {
        let content = if let Some(img_b64) = image {
            MessageContent::Complex(vec![
                ContentPart::Text {
                    text: prompt.to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: if img_b64.starts_with("data:") {
                            img_b64.to_string()
                        } else {
                            format!("data:image/png;base64,{}", img_b64)
                        },
                    },
                },
            ])
        } else {
            MessageContent::Text(prompt.to_string())
        };

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content,
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
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse detailed error from JSON
            let error_msg = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(msg) = json["error"]["message"].as_str() {
                    msg.to_string()
                } else {
                    body
                }
            } else {
                body
            };

            return Err(anyhow!("API Error ({}): {}", status, error_msg));
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
