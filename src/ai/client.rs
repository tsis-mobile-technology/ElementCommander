use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;

pub struct AiClient {
    base_url: String,
    model: String,
    client: Client,
}

#[derive(Debug, serde::Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, serde::Deserialize)]
struct ChatMessage {
    content: String,
}

impl AiClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: Client::new(),
        }
    }

    pub async fn summarize_file(&self, file_content: String, file_path: String) -> Result<String> {
        let prompt = format!(
            "파일 경로: {}\n\n다음 파일의 내용을 간단하게 요약해줘. 핵심만 2-3줄로 설명해줘.\n\n{}",
            file_path, file_content
        );

        self.query(&prompt).await
    }

    pub async fn query(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "top_p": 0.9,
            "max_tokens": 512,
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("AI API 요청 실패: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("AI API 오류 ({}): {}", status, error_text));
        }

        let data: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("응답 파싱 실패: {}", e))?;

        data.choices
            .first()
            .ok_or_else(|| anyhow!("응답에 내용이 없습니다"))
            .map(|choice| choice.message.content.clone())
    }
}
