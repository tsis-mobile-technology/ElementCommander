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
            "파일 경로: {}\n\n다음 코드 파일을 분석하고 아래 형식으로 정리해줘:\n\n**주요 기능:**\n- 이 파일의 주요 목적과 기능 (1-2줄)\n\n**공개 API/함수:**\n- 외부에서 사용 가능한 주요 함수나 클래스 목록\n\n**핵심 코드:**\n- 주요 로직이나 알고리즘 (간단하게)\n\n**의존성:**\n- 이 파일이 사용하는 외부 모듈이나 라이브러리\n\n다음 코드:\n\n{}",
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

    pub async fn analyze_image(&self, file_info: String, file_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "파일 경로: {}\n\n이 이미지 파일의 메타데이터와 속성을 분석해줘. 다음 정보를 정리해서 제시해줘:\n\n**파일 정보:**\n- 파일명\n- 파일 크기\n- 수정 날짜\n\n**예상 이미지 타입:**\n- 파일 확장자를 기반으로 한 이미지 형식\n\n**주의사항:**\n- 이미지 파일의 성격이나 특징 (크기, 형식 등)\n\n파일 정보:\n{}",
            file_path, file_info
        );

        self.query(&prompt).await
    }

    pub async fn analyze_folder(&self, folder_info: String, folder_path: String) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "폴더 경로: {}\n\n다음 폴더 구조를 분석하고 아래 형식으로 정리해줘:\n\n**프로젝트 타입:**\n- 이것이 어떤 종류의 프로젝트인지 (웹, 모바일, 라이브러리 등)\n\n**용도:**\n- 이 프로젝트의 주요 목적이나 기능\n\n**주요 컴포넌트:**\n- 폴더 구조에서 보이는 중요한 디렉토리나 파일들\n\n**특징:**\n- 이 프로젝트만의 특징이나 구조적 특징\n\n폴더 정보:\n{}",
            folder_path, folder_info
        );

        self.query(&prompt).await
    }

    pub async fn interpret_command(
        &self,
        nl_command: &str,
        current_dir: &str,
        file_listing: &str,
    ) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "You are a file manager. Return ONLY a JSON array. No other text.\n\nCurrent directory: {}\n\nFiles in this directory:\n{}\n\nUser request: {}\n\nRespond ONLY with this JSON format (no explanations, no markdown, just the array):\n[\n  {{\"op\": \"delete\", \"path\": \"/absolute/path/to/file\"}},\n  {{\"op\": \"move\", \"from\": \"/absolute/path/from\", \"to\": \"/absolute/path/to\"}},\n  {{\"op\": \"copy\", \"from\": \"/absolute/path/from\", \"to\": \"/absolute/path/to\"}},\n  {{\"op\": \"mkdir\", \"path\": \"/absolute/path/to/new/directory\"}},\n  {{\"op\": \"rename\", \"from\": \"/absolute/path/to/file\", \"to\": \"newname\"}}\n]\n\nRules:\n1. All paths must be absolute\n2. All paths must be inside {}\n3. Only list files that exist in the directory above\n4. Return empty array [] if no matching files\n5. Return ONLY the JSON array, nothing else",
            current_dir, file_listing, nl_command, current_dir
        );

        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "top_p": 0.9,
            "max_tokens": 1024,
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

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("AI API 오류: {} - {}", status, error_text);
            return Err(anyhow!("AI API 오류 ({}): {}", status, error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("응답 파싱 실패: {}", e);
                anyhow!("응답 파싱 실패: {}", e)
            })?;

        let choice = data
            .get("choices")
            .and_then(|c| c.get(0))
            .ok_or_else(|| anyhow!("응답에 내용이 없습니다"))?;

        let content = choice
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let thinking = choice
            .get("message")
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        tracing::debug!("AI 명령 해석 응답: {} 글자", content.len());
        tracing::debug!("AI 응답 내용: {}", content);

        Ok(crate::ai::AiResponse::new(thinking, content.to_string()))
    }

    pub async fn batch_rename(
        &self,
        pattern: &str,
        current_dir: &str,
        file_list: &str,
    ) -> Result<crate::ai::AiResponse> {
        let prompt = format!(
            "You are a file manager. Return ONLY a JSON array. No other text.\n\nCurrent directory: {}\n\nFiles in this directory:\n{}\n\nUser request: Apply this rename pattern to ALL files above: {}\n\nRespond ONLY with this JSON format (no explanations, no markdown, just the array):\n[\n  {{\"op\": \"rename\", \"from\": \"/absolute/path/to/file.ext\", \"to\": \"new_name.ext\"}}\n]\n\nRules:\n1. All paths must be absolute (start with /)\n2. All paths must be inside {}\n3. Only list files that exist in the directory above\n4. 'to' is ONLY the new filename, no path\n5. Process EVERY file in the list\n6. Return ONLY the JSON array, nothing else",
            current_dir, file_list, pattern, current_dir
        );

        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "top_p": 0.9,
            "max_tokens": 1024,
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

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("AI API 오류: {} - {}", status, error_text);
            return Err(anyhow!("AI API 오류 ({}): {}", status, error_text));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("응답 파싱 실패: {}", e);
                anyhow!("응답 파싱 실패: {}", e)
            })?;

        let choice = data
            .get("choices")
            .and_then(|c| c.get(0))
            .ok_or_else(|| anyhow!("응답에 내용이 없습니다"))?;

        let content = choice
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let thinking = choice
            .get("message")
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        tracing::debug!("AI 배치 리네이밍 응답: {} 글자", content.len());
        tracing::debug!("AI 응답 내용: {}", content);

        Ok(crate::ai::AiResponse::new(thinking, content.to_string()))
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
        let paragraphs: Vec<&str> = thinking_text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        if paragraphs.is_empty() {
            return "응답을 받지 못했습니다".to_string();
        }

        // 최종 분석 섹션을 찾기 위해 역순으로 스캔
        // "최종 분석", "분석 결과", "결론" 등의 키워드를 찾음
        let mut final_section_start = paragraphs.len();
        for (i, paragraph) in paragraphs.iter().enumerate() {
            let lower = paragraph.to_lowercase();
            if lower.contains("최종") || lower.contains("결론") || lower.contains("분석 결과") || lower.contains("정리하면") {
                final_section_start = i;
                break;
            }
        }

        // 최종 섹션이 발견된 경우, 그 이후의 모든 내용을 수집
        if final_section_start < paragraphs.len() {
            let result_parts: Vec<&str> = paragraphs[final_section_start..]
                .iter()
                .filter(|p| {
                    let p = p.trim();
                    // 메타정보 필터링 (사고 과정 같은 것 제외)
                    !p.starts_with("다시") && !p.starts_with("생각") && !p.starts_with("확인") &&
                    !p.starts_with("이제") && !p.starts_with("따라서 다시") && !p.is_empty()
                })
                .map(|s| s.trim())
                .collect();

            if !result_parts.is_empty() {
                return result_parts.join("\n\n");
            }
        }

        // 최종 섹션을 찾지 못한 경우, 형식화된 답변 찾기
        // "**" 또는 "-"로 시작하는 구조화된 항목들을 찾음
        let mut structured_parts = Vec::new();
        for paragraph in paragraphs.iter() {
            let p = paragraph.trim();
            if p.starts_with("**") || p.starts_with("##") || p.starts_with("-") || p.starts_with("•") {
                structured_parts.push(p);
            }
        }

        if !structured_parts.is_empty() {
            return structured_parts.join("\n");
        }

        // 마지막 수단: 역순으로 긴 문단 찾기 (메타정보 제외)
        for i in (0..paragraphs.len()).rev() {
            let paragraph = paragraphs[i].trim();
            let lower = paragraph.to_lowercase();

            // 메타정보나 생각 과정 필터링
            if !paragraph.starts_with("*") &&
               !lower.contains("다시 생각") &&
               !lower.contains("정정합니다") &&
               !lower.contains("아, ") &&
               !lower.contains("잠깐,") &&
               !paragraph.is_empty() &&
               paragraph.len() > 20 {
                return paragraph.to_string();
            }
        }

        // 최후의 수단: 마지막 문단
        paragraphs.last()
            .map(|p| p.trim().to_string())
            .unwrap_or_else(|| "응답을 받지 못했습니다".to_string())
    }
}
