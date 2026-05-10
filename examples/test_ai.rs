use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();
    let base_url = "http://localhost:8080/v1";

    println!("Testing improved AI analysis...\n");

    // Test code structure analysis
    println!("=== Code Structure Analysis ===");
    let code = "pub fn factorial(n: u32) -> u32 { match n { 0|1 => 1, _ => n * factorial(n-1) } }\npub fn add(a: i32, b: i32) -> i32 { a + b }";
    
    let prompt = format!("파일 경로: math.rs\n\n다음 코드 파일을 분석하고 아래 형식으로 정리해줘:\n\n**주요 기능:**\n- 이 파일의 주요 목적과 기능 (1-2줄)\n\n**공개 API/함수:**\n- 외부에서 사용 가능한 주요 함수나 클래스 목록\n\n**핵심 코드:**\n- 주요 로직이나 알고리즘 (간단하게)\n\n다음 코드:\n\n{}", code);

    let body = json!({
        "model": "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.7,
        "max_tokens": 300,
    });

    if let Ok(response) = client.post(&format!("{}/chat/completions", base_url)).json(&body).send().await {
        if let Ok(data) = response.json::<serde_json::Value>().await {
            if let Some(content) = data["choices"][0]["message"]["content"].as_str() {
                if !content.is_empty() {
                    println!("Content:\n{}\n", content);
                } else {
                    println!("Content field is empty, checking reasoning_content...\n");
                }
            }
            if let Some(reasoning) = data["choices"][0]["message"]["reasoning_content"].as_str() {
                println!("Reasoning length: {} chars\n", reasoning.len());
            }
        }
    }

    println!("\n=== Image Metadata Analysis ===");
    let img_info = "파일명: photo.jpg\n크기: 2048576 bytes\n수정일: 2026-05-10";
    
    let prompt2 = format!("파일 경로: photo.jpg\n\n이 이미지 파일의 메타데이터와 속성을 분석해줘. 다음 정보를 정리해서 제시해줘:\n\n**파일 정보:**\n- 파일명\n- 파일 크기\n- 수정 날짜\n\n**예상 이미지 타입:**\n- 파일 확장자를 기반으로 한 이미지 형식\n\n파일 정보:\n{}", img_info);

    let body2 = json!({
        "model": "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf",
        "messages": [{"role": "user", "content": prompt2}],
        "temperature": 0.7,
        "max_tokens": 300,
    });

    if let Ok(response) = client.post(&format!("{}/chat/completions", base_url)).json(&body2).send().await {
        if let Ok(data) = response.json::<serde_json::Value>().await {
            if let Some(content) = data["choices"][0]["message"]["content"].as_str() {
                if !content.is_empty() {
                    println!("Content:\n{}\n", content);
                } else {
                    println!("Content field is empty\n");
                }
            }
        }
    }

    Ok(())
}
