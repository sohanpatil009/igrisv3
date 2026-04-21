// src/core/gemini.rs
// Gemini API integration for enhanced web search and STT

use serde::{Deserialize, Serialize};
use std::time::Duration;
use reqwest::Client;
use anyhow::{Result, anyhow};

const GEMINI_API_KEY: &str = "AIzaSyAb8RN6LMMBsptZQiH_9ns_H6ns2oSOyKIFs20xPtM6Yg7rqdIg";
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

pub struct GeminiClient {
    client: Client,
}

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Check if internet connection is available
    pub async fn is_online() -> bool {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        
        // Try to reach Google's DNS
        match client.get("https://8.8.8.8").send().await {
            Ok(_) => true,
            Err(_) => {
                // Fallback: try to reach Gemini API directly
                match client.get("https://generativelanguage.googleapis.com").send().await {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
        }
    }

    /// Enhanced web search using Gemini
    pub async fn smart_web_search(&self, query: &str) -> Result<String> {
        let prompt = format!(
            "You are a helpful assistant. The user is asking: '{}'

Please provide a direct, concise answer to their question. If it's a factual question, provide the facts. If it's a how-to question, provide step-by-step instructions. Keep the response under 200 words and make it conversational for voice output.

If you don't have current information, suggest what the user should search for or where to find the information.",
            query
        );

        self.generate_response(&prompt).await
    }

    /// Process voice command with better understanding
    pub async fn enhance_voice_command(&self, command: &str) -> Result<String> {
        let prompt = format!(
            "You are IGRIS, a voice assistant. The user said: '{}'

Analyze this command and respond with ONE of these formats:

1. If it's a clear system command (open app, close app, etc.), respond with:
   ACTION: [action_type]:[target]
   Example: ACTION: open_app:chrome

2. If it's a question or needs information, respond with:
   ANSWER: [your helpful response]

3. If it's unclear, respond with:
   CLARIFY: [ask for clarification]

Keep responses concise and natural for voice output.",
            command
        );

        self.generate_response(&prompt).await
    }

    /// Generate response using Gemini API
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: 0.7,
                top_k: 40,
                top_p: 0.95,
                max_output_tokens: 1024,
            },
        };

        let url = format!(
            "{}/models/gemini-1.5-flash:generateContent?key={}",
            GEMINI_BASE_URL, GEMINI_API_KEY
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err(anyhow!("No response from Gemini"))
    }
}

/// Enhanced web search that uses Gemini for direct answers
pub async fn enhanced_web_search(query: &str) -> Option<String> {
    // Check if online
    if !GeminiClient::is_online().await {
        return None;
    }

    let client = GeminiClient::new();
    match client.smart_web_search(query).await {
        Ok(response) => Some(response),
        Err(e) => {
            eprintln!("Gemini web search error: {}", e);
            None
        }
    }
}

/// Enhanced voice command processing
pub async fn enhance_voice_command(command: &str) -> Option<String> {
    // Check if online
    if !GeminiClient::is_online().await {
        return None;
    }

    let client = GeminiClient::new();
    match client.enhance_voice_command(command).await {
        Ok(response) => Some(response),
        Err(e) => {
            eprintln!("Gemini voice enhancement error: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connectivity() {
        let is_online = GeminiClient::is_online().await;
        println!("Internet connectivity: {}", is_online);
    }

    #[tokio::test]
    async fn test_web_search() {
        if let Some(response) = enhanced_web_search("What is Rust programming language?").await {
            println!("Gemini response: {}", response);
            assert!(!response.is_empty());
        }
    }
}