use serde_json::{json, Value};
use thiserror::Error;
use tracing::debug;

use crate::{config::SYSTEM_PROMPT, db::Memory};

#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Gemini API error (HTTP {status}): {body}")]
    ApiError { status: u16, body: String },

    #[error("Unexpected response shape: {0}")]
    BadResponse(String),
}

/// Wraps reqwest and talks to the Gemini generateContent endpoint.
#[derive(Clone)]
pub struct GeminiClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(http: reqwest::Client, api_key: String, model: String) -> Self {
        Self { http, api_key, model }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Send history + new message to Gemini and return the reply text.
    pub async fn chat(&self, history: &[Memory], new_text: &str) -> Result<String, GeminiError> {
        let mut contents: Vec<Value> = history
            .iter()
            .map(|m| json!({ "role": m.role, "parts": [{ "text": m.text }] }))
            .collect();
        contents.push(json!({ "role": "user", "parts": [{ "text": new_text }] }));

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let body = json!({
            "system_instruction": { "parts": [{ "text": SYSTEM_PROMPT }] },
            "contents": contents,
        });

        debug!(model = self.model, turns = contents.len(), "Sending to Gemini");

        let resp = self
            .http
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let data: Value = resp.json().await?;

        if !status.is_success() {
            return Err(GeminiError::ApiError { status: status.as_u16(), body: data.to_string() });
        }

        // Join all text parts from the first candidate.
        let parts = data["candidates"][0]["content"]["parts"]
            .as_array()
            .ok_or_else(|| GeminiError::BadResponse(format!("No parts: {data}")))?;

        let text: String = parts
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join("");

        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(GeminiError::BadResponse(format!("Empty text: {data}")));
        }

        Ok(text)
    }
}
