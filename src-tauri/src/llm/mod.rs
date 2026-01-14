use crate::config::LlmProvider;
use serde::{Deserialize, Serialize};

/// LLM API client for text refinement (supports Ollama and OpenAI-compatible APIs)
pub struct LlmClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
    provider: LlmProvider,
}

// Ollama API structures
#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

// OpenAI-compatible API structures
#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(base_url: &str, model: &str, provider: LlmProvider) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            provider,
        }
    }

    /// Create a client with default settings (Ollama)
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self::new("http://localhost:11434", "gpt-oss:20b", LlmProvider::Ollama)
    }

    /// Refine transcribed text using LLM with custom prompt template
    pub async fn refine_text_with_prompt(
        &self,
        raw_text: &str,
        prompt_template: &str,
    ) -> Result<String, String> {
        let prompt = prompt_template.replace("{input}", raw_text);

        match self.provider {
            LlmProvider::Ollama => self.refine_with_ollama(&prompt).await,
            LlmProvider::OpenAICompat => self.refine_with_openai_compat(&prompt).await,
        }
    }

    async fn refine_with_ollama(&self, prompt: &str) -> Result<String, String> {
        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        tracing::info!("Sending request to Ollama: {}", url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Ollama API error ({}): {}", status, body));
        }

        let result: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let refined = result.response.trim().to_string();
        tracing::info!("LLM refinement complete (Ollama)");

        Ok(refined)
    }

    async fn refine_with_openai_compat(&self, prompt: &str) -> Result<String, String> {
        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);
        tracing::info!("Sending request to OpenAI-compatible API: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to OpenAI-compatible API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI-compatible API error ({}): {}", status, body));
        }

        let result: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI-compatible response: {}", e))?;

        let refined = result
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default();

        tracing::info!("LLM refinement complete (OpenAI-compatible)");

        Ok(refined)
    }

    /// Check if LLM server is available
    pub async fn is_available(&self) -> bool {
        let url = match self.provider {
            LlmProvider::Ollama => format!("{}/api/tags", self.base_url),
            LlmProvider::OpenAICompat => format!("{}/v1/models", self.base_url),
        };

        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get the model name
    #[allow(dead_code)]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the base URL
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the provider
    #[allow(dead_code)]
    pub fn provider(&self) -> &LlmProvider {
        &self.provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_client() {
        let client = LlmClient::default();
        assert_eq!(client.base_url(), "http://localhost:11434");
        assert_eq!(client.model(), "gpt-oss:20b");
        assert_eq!(client.provider(), &LlmProvider::Ollama);
    }

    #[test]
    fn test_ollama_client() {
        let client = LlmClient::new("http://192.168.1.100:11434/", "llama2", LlmProvider::Ollama);
        assert_eq!(client.base_url(), "http://192.168.1.100:11434");
        assert_eq!(client.model(), "llama2");
        assert_eq!(client.provider(), &LlmProvider::Ollama);
    }

    #[test]
    fn test_openai_compat_client() {
        let client = LlmClient::new("http://localhost:1234/", "qwen2.5", LlmProvider::OpenAICompat);
        assert_eq!(client.base_url(), "http://localhost:1234");
        assert_eq!(client.model(), "qwen2.5");
        assert_eq!(client.provider(), &LlmProvider::OpenAICompat);
    }
}
