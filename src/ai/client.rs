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
    #[serde(default)]
    content: String,
    #[serde(default)]
    reasoning_content: Option<String>,
}

impl AiClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: Client::new(),
        }
    }

    pub async fn summarize_file(&self, file_content: String, file_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "파일 경로: {}\n\n다음 파일의 내용을 간단하게 요약해줘. 핵심만 2-3줄로 설명해줘.\n\n{}",
            file_path, file_content
        );

        self.query(&prompt).await
    }

    pub async fn scan_security(&self, file_content: String, file_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "파일: {}\n\n다음 파일에서 민감한 정보(API 키, 비밀번호, 개인정보 등)를 찾아줘. 패턴과 위치를 명확히 지적해줘.\n\n{}",
            file_path, file_content
        );

        self.query(&prompt).await
    }

    pub async fn analyze_code(&self, file_content: String, file_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "파일: {}\n\n다음 코드 파일의 구조를 분석해줘. 주요 함수/클래스, 공개 API, 의존성을 간단히 정리해줘.\n\n{}",
            file_path, file_content
        );

        self.query(&prompt).await
    }

    pub async fn compare_files(&self, file1_content: String, file1_path: String, file2_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "파일 1: {}\n파일 2: {}\n\n두 파일의 차이점을 분석해줘. 무엇이 변경되었고 왜 변경된 것 같은지 설명해줘.\n\n=== 파일 1 내용 ===\n{}\n\n=== 파일 2 내용 ===\n(파일 2가 제공되지 않았으므로 파일 1 기준으로만 제시됨)",
            file1_path, file2_path, file1_content
        );

        self.query(&prompt).await
    }

    pub async fn analyze_folder(&self, folder_info: String, folder_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "폴더: {}\n\n다음 폴더 구조와 파일 목록을 분석해줘. 프로젝트 타입은 무엇이고 어떤 용도인지, 주요 컴포넌트는 무엇인지 설명해줘.\n\n{}",
            folder_path, folder_info
        );

        self.query(&prompt).await
    }

    pub async fn query(&self, prompt: &str) -> Result<crate::ai::AiResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        tracing::info!("AI 요청 시작: {}", url);
        tracing::debug!("프롬프트 길이: {} 글자", prompt.len());

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
            .map_err(|e| {
                tracing::error!("AI API 요청 실패: {}", e);
                anyhow!("AI API 요청 실패: {}", e)
            })?;

        tracing::info!("AI 응답 상태: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("AI API 오류: {} - {}", status, error_text);
            return Err(anyhow!("AI API 오류 ({}): {}", status, error_text));
        }

        let data: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("응답 파싱 실패: {}", e);
                anyhow!("응답 파싱 실패: {}", e)
            })?;

        tracing::debug!("응답 선택지 수: {}", data.choices.len());

        let choice = data
            .choices
            .first()
            .ok_or_else(|| {
                tracing::error!("응답에 내용이 없습니다");
                anyhow!("응답에 내용이 없습니다")
            })?;

        // thinking_content (CoT 모델)와 content 분리
        let thinking = choice.message.reasoning_content.clone();

        // content가 비어있으면 reasoning_content에서 최종 결과 추출
        let result = if choice.message.content.is_empty() {
            // reasoning_content에서 최종 결과 추출 (보통 마지막 의미 있는 문단)
            if let Some(ref thinking_text) = thinking {
                Self::extract_final_result(thinking_text)
            } else {
                "응답을 받지 못했습니다".to_string()
            }
        } else {
            choice.message.content.clone()
        };

        tracing::info!("AI 응답 수신 - thinking: {:?} 글자, result: {} 글자",
            thinking.as_ref().map(|t| t.len()),
            result.len());

        Ok(crate::ai::AiResponse::new(thinking, result))
    }

    fn extract_final_result(thinking_text: &str) -> String {
        // 역순으로 빈 줄을 기준으로 문단을 찾음
        let paragraphs: Vec<&str> = thinking_text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        if paragraphs.is_empty() {
            return "응답을 받지 못했습니다".to_string();
        }

        // 마지막 문단부터 찾아서, 실제 결과 같은 것을 찾음
        for i in (0..paragraphs.len()).rev() {
            let paragraph = paragraphs[i].trim();

            // 숫자로 시작하거나, 실제 결과 같은 문단 찾음
            if !paragraph.starts_with("*") && !paragraph.is_empty() {
                return paragraph.to_string();
            }
        }

        // 모두 메타정보면 전체 마지막 문단 사용
        paragraphs.last()
            .map(|p| p.trim().to_string())
            .unwrap_or_else(|| "응답을 받지 못했습니다".to_string())
    }
}
