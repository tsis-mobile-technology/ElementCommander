use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ai::AiCommandState;
use crate::commands::PlannedOp;

pub fn render_ai_command_confirm(frame: &mut Frame, ai_command_state: &AiCommandState) {
    let area = frame.area();

    // 레이아웃: 제목 + 내용 + 상태바
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),      // 제목
            Constraint::Min(5),         // 작업 목록
            Constraint::Length(3),      // 상태바
        ])
        .split(area);

    // 제목
    let title_block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Cyan).bold())
        .title("🤖 AI Commander - 실행 예정 작업 목록")
        .title_alignment(Alignment::Center);
    frame.render_widget(title_block, chunks[0]);

    let title_inner = chunks[0];
    let title_text = Paragraph::new("작업을 검토하고 [Y] 실행 또는 [N] 취소")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(title_text, title_inner);

    // 작업 목록
    let mut ops_text = Vec::new();
    for (idx, op) in ai_command_state.ops.iter().enumerate() {
        let full_line = match op {
            PlannedOp::Delete { path } => {
                format!("🗑️  삭제: {}", path.display())
            }
            PlannedOp::Move { from, to } => {
                format!("📦 이동: {} → {}", from.display(), to.display())
            }
            PlannedOp::Copy { from, to } => {
                format!("📄 복사: {} → {}", from.display(), to.display())
            }
            PlannedOp::Mkdir { path } => {
                format!("📁 폴더 생성: {}", path.display())
            }
            PlannedOp::Rename { from, to } => {
                format!("✏️  이름 변경: {} → {}", from.display(), to)
            }
        };

        let style = if idx == ai_command_state.scroll {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let line_content = if idx == ai_command_state.scroll {
            format!("→ {}", full_line)
        } else {
            format!("  {}", full_line)
        };

        ops_text.push(Line::from(Span::styled(line_content, style)));
    }

    let ops_list = Paragraph::new(ops_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .style(Style::default().bg(Color::Rgb(15, 15, 20))),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(ops_list, chunks[1]);

    // 상태바
    let status_text = format!(
        "[Y] 실행  [N] 취소  [↑↓] 스크롤  총 {} 개 작업  선택: {} / {}",
        ai_command_state.ops.len(),
        ai_command_state.scroll + 1,
        ai_command_state.ops.len()
    );

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(status, chunks[2]);
}
