fn extract_final_result(thinking_text: &str) -> String {
    let paragraphs: Vec<&str> = thinking_text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    if paragraphs.is_empty() {
        return "응답을 받지 못했습니다".to_string();
    }

    // 최종 분석 섹션을 찾기 위해 역순으로 스캔
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

    // 마지막 수단: 역순으로 긴 문단 찾기
    for i in (0..paragraphs.len()).rev() {
        let paragraph = paragraphs[i].trim();
        let lower = paragraph.to_lowercase();

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

    paragraphs.last()
        .map(|p| p.trim().to_string())
        .unwrap_or_else(|| "응답을 받지 못했습니다".to_string())
}

fn main() {
    // Sample thinking text from Qwen CoT model
    let sample = r#"사용자가 이미지 파일의 메타데이터를 분석해달라고 요청했습니다.

주어진 정보:
- 파일명: photo.jpg
- 크기: 2048576 bytes (약 2MB)
- 수정일: 2026-05-10

이 정보를 바탕으로 분석하겠습니다:

1. 파일명이 photo.jpg라는 것은 일반적인 사진 파일입니다.
2. 크기가 2MB라는 것은 중간 정도의 고해상도 사진입니다.
3. 확장자가 jpg라는 것은 JPEG 형식의 압축 이미지입니다.

최종 분석:

**파일 정보:**
- 파일명: photo.jpg
- 파일 크기: 2048576 bytes (약 2MB)
- 수정 날짜: 2026-05-10

**예상 이미지 타입:**
- JPEG 형식의 디지털 사진 (jpg 확장자 기반)

**주의사항:**
- 약 2MB 크기의 중간 정도 고해상도 이미지 파일
- JPEG 형식으로 압축되어 있어 원본 해상도보다 용량이 작을 수 있음"#;

    println!("Original thinking length: {} chars\n", sample.len());
    let result = extract_final_result(sample);
    println!("Extracted result:\n{}\n", result);
    println!("Extracted result length: {} chars", result.len());
}
