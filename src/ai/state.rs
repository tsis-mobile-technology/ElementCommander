use crate::ui::viewer::LineContent;
use ratatui::style::Color;
use textwrap::wrap;

pub struct AiState {
    pub lines: Vec<LineContent>,
    pub scroll: u16,
    pub is_loading: bool,
    pub query: String,
}

impl AiState {
    pub fn new(response: String) -> Self {
        let lines = Self::format_response(&response);
        Self {
            lines,
            scroll: 0,
            is_loading: false,
            query: String::new(),
        }
    }

    pub fn loading(query: String) -> Self {
        Self {
            lines: vec![Self::create_line("⏳ AI 처리 중...")],
            scroll: 0,
            is_loading: true,
            query,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            lines: vec![Self::create_line(&format!("❌ 오류: {}", error))],
            scroll: 0,
            is_loading: false,
            query: String::new(),
        }
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
}
