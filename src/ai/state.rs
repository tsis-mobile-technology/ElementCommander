use crate::ui::viewer::LineContent;
use crate::ai::AiResponse;
use crate::commands::PlannedOp;
use ratatui::style::Color;
use textwrap::wrap;

pub struct AiState {
    pub lines: Vec<LineContent>,
    pub scroll: u16,
    pub is_loading: bool,
    pub query: String,
    pub show_thinking: bool,  // thinking process 표시 여부
    pub thinking_lines: Vec<LineContent>,  // thinking process 섹션
    pub result_lines: Vec<LineContent>,    // 최종 결과 섹션
}

impl AiState {
    pub fn new(response: AiResponse) -> Self {
        let mut all_lines = Vec::new();
        let mut thinking_lines = Vec::new();
        let mut result_lines = Vec::new();

        // Thinking process (접을 수 있음)
        if let Some(thinking) = &response.thinking {
            if !thinking.is_empty() {
                thinking_lines.push(Self::create_line("💭 Thinking Process (T로 토글)"));
                thinking_lines.push(Self::create_line(&"─".repeat(80)));
                thinking_lines.extend(Self::format_response(thinking));
                thinking_lines.push(Self::create_line(&"─".repeat(80)));

                all_lines.extend(thinking_lines.clone());
            }
        }

        // 최종 결과 (강조)
        result_lines.push(Self::create_line(""));
        result_lines.push(Self::create_line_bright("✨ 분석 결과"));
        result_lines.extend(Self::format_response(&response.result));

        all_lines.extend(result_lines.clone());

        Self {
            lines: all_lines,
            scroll: 0,
            is_loading: false,
            query: String::new(),
            show_thinking: response.thinking.is_some(),  // 기본값: thinking이 있으면 표시
            thinking_lines,
            result_lines,
        }
    }

    pub fn loading(query: String) -> Self {
        Self {
            lines: vec![Self::create_line("⏳ AI 처리 중...")],
            scroll: 0,
            is_loading: true,
            query,
            show_thinking: false,
            thinking_lines: Vec::new(),
            result_lines: Vec::new(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            lines: vec![Self::create_line(&format!("❌ 오류: {}", error))],
            scroll: 0,
            is_loading: false,
            query: String::new(),
            show_thinking: false,
            thinking_lines: Vec::new(),
            result_lines: Vec::new(),
        }
    }

    pub fn toggle_thinking(&mut self) {
        self.show_thinking = !self.show_thinking;
        self.rebuild_lines();
    }

    fn rebuild_lines(&mut self) {
        let mut lines = Vec::new();

        if self.show_thinking && !self.thinking_lines.is_empty() {
            lines.extend(self.thinking_lines.clone());
        }
        lines.extend(self.result_lines.clone());

        self.lines = lines;
        // 스크롤 위치 유지 (필요시 조정)
        self.scroll = self.scroll.min((self.lines.len() as u16).saturating_sub(20));
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_lines: u16) {
        if self.scroll + max_lines < self.lines.len() as u16 {
            self.scroll += 1;
        }
    }

    pub fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(10);
    }

    pub fn page_down(&mut self, max_lines: u16) {
        self.scroll = (self.scroll + 10).min((self.lines.len() as u16).saturating_sub(max_lines));
    }

    fn format_response(text: &str) -> Vec<LineContent> {
        let lines: Vec<String> = text
            .lines()
            .flat_map(|line| {
                if line.is_empty() {
                    vec![String::new()]
                } else {
                    wrap(line, 120)
                        .iter()
                        .map(|s| s.to_string())
                        .collect()
                }
            })
            .collect();

        lines.into_iter().map(|s| Self::create_line(&s)).collect()
    }

    fn create_line(text: &str) -> LineContent {
        LineContent {
            raw: text.to_string(),
            styled: vec![(Color::White, text.to_string(), false)],
        }
    }

    fn create_line_bright(text: &str) -> LineContent {
        LineContent {
            raw: text.to_string(),
            styled: vec![(Color::Cyan, text.to_string(), true)],  // Cyan + Bold
        }
    }
}

pub struct AiCommandState {
    pub ops: Vec<PlannedOp>,
    pub scroll: usize,
}

impl AiCommandState {
    pub fn new(ops: Vec<PlannedOp>) -> Self {
        Self { ops, scroll: 0 }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.ops.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }
}
