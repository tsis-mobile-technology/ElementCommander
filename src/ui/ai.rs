use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ai::AiState;

pub fn render_ai_mode(frame: &mut Frame, ai_state: &AiState) {
    let area = frame.size();

    // 상단 타이틀
    let title_height = 1;
    let content_height = area.height.saturating_sub(title_height + 1);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Min(content_height),
            Constraint::Length(1),
        ])
        .split(area);

    // 타이틀 바
    let title_block = Block::default()
        .title(" 🤖 AI 분석 결과 ")
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::BOTTOM)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let title_para = Paragraph::new("");
    frame.render_widget(title_para.block(title_block), chunks[0]);

    // 콘텐츠 영역
    let content_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let visible_lines: Vec<Line> = ai_state
        .lines
        .iter()
        .skip(ai_state.scroll as usize)
        .take(content_height as usize)
        .map(|line| {
            // LineContent의 styled 정보 사용
            if !line.styled.is_empty() {
                let spans: Vec<Span> = line
                    .styled
                    .iter()
                    .map(|(color, text, bold)| {
                        let style = if *bold {
                            Style::default()
                                .fg(*color)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(*color)
                        };
                        Span::styled(text.clone(), style)
                    })
                    .collect();
                Line::from(spans)
            } else {
                Line::from(Span::raw(line.raw.clone()))
            }
        })
        .collect();

    let content = Paragraph::new(visible_lines)
        .block(content_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(content, chunks[1]);

    // 하단 상태바
    let status_text = if ai_state.is_loading {
        "⏳ 처리 중... (ESC/q로 취소)".to_string()
    } else if ai_state.lines.first().map(|l| l.raw.starts_with("❌")).unwrap_or(false) {
        format!(
            "❌ 오류 발생 (↑↓ 스크롤, PgUp/PgDn 페이지, ESC/q 닫기)",
        )
    } else {
        let thinking_indicator = if ai_state.show_thinking {
            "💭 ON"
        } else {
            "💭 OFF"
        };
        format!(
            "✓ 스크롤: ↑↓ | 페이지: PgUp/PgDn | Thinking: T | 닫기: ESC/q  [{}] ({}/{})",
            thinking_indicator,
            ai_state.scroll + 1,
            (ai_state.lines.len() as u16).max(1)
        )
    };

    let status_para = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray).add_modifier(Modifier::DIM));
    frame.render_widget(status_para, chunks[2]);
}
