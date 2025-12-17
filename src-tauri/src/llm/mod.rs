use serde::{Deserialize, Serialize};

/// Ollama API client for LLM text refinement
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

const DEFAULT_PROMPT_TEMPLATE: &str = r#"以下の音声認識結果を自然な日本語に整形してください。
誤字脱字の修正、句読点の追加、文法の修正を行ってください。
整形後のテキストのみを出力してください。余計な説明は不要です。

入力: {input}

出力:"#;

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    /// Create a client with default settings
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self::new("http://localhost:11434", "gpt-oss:20b")
    }

    /// Refine transcribed text using LLM
    pub async fn refine_text(&self, raw_text: &str) -> Result<String, String> {
        self.refine_text_with_prompt(raw_text, DEFAULT_PROMPT_TEMPLATE)
            .await
    }

    /// Refine transcribed text using LLM with custom prompt template
    pub async fn refine_text_with_prompt(
        &self,
        raw_text: &str,
        prompt_template: &str,
    ) -> Result<String, String> {
        let prompt = prompt_template.replace("{input}", raw_text);

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt,
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

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let refined = result.response.trim().to_string();
        tracing::info!("LLM refinement complete: {} -> {}", raw_text, refined);

        Ok(refined)
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_client() {
        let client = OllamaClient::default();
        assert_eq!(client.base_url(), "http://localhost:11434");
        assert_eq!(client.model(), "gpt-oss:20b");
    }

    #[test]
    fn test_custom_client() {
        let client = OllamaClient::new("http://192.168.1.100:11434/", "llama2");
        assert_eq!(client.base_url(), "http://192.168.1.100:11434");
        assert_eq!(client.model(), "llama2");
    }
}
