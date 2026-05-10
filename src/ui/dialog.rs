use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

#[derive(Clone, Debug)]
pub enum DialogKind {
    Copy,
    Move,
    Mkdir,
    Delete,
    Rename,
    Find,
    Pack,
    AiCommand,
    BatchRename,
    AddNote,
    GenerateScript,
}

#[derive(Clone, Debug)]
pub struct DialogState {
    pub kind: DialogKind,
    pub input: String,
    pub cursor: usize,
    pub message: String,
    pub error: Option<String>,
}

impl DialogState {
    pub fn new_copy(selected_count: usize, default_path: String) -> Self {
        DialogState {
            kind: DialogKind::Copy,
            input: default_path.clone(),
            cursor: default_path.len(),
            message: format!("📋 {} 개 항목을 복사합니다", selected_count),
            error: None,
        }
    }

    pub fn new_move(selected_count: usize, default_path: String) -> Self {
        DialogState {
            kind: DialogKind::Move,
            input: default_path.clone(),
            cursor: default_path.len(),
            message: format!("➡️  {} 개 항목을 이동합니다", selected_count),
            error: None,
        }
    }

    pub fn new_pack(selected_count: usize, default_name: String) -> Self {
        let cursor = default_name.len();
        DialogState {
            kind: DialogKind::Pack,
            input: default_name,
            cursor,
            message: format!("📦 {} 개 항목을 압축합니다", selected_count),
            error: None,
        }
    }

    pub fn new_mkdir() -> Self {
        DialogState {
            kind: DialogKind::Mkdir,
            input: String::new(),
            cursor: 0,
            message: "📁 새 디렉토리 생성".to_string(),
            error: None,
        }
    }

    pub fn new_delete(selected_count: usize) -> Self {
        DialogState {
            kind: DialogKind::Delete,
            input: String::new(),
            cursor: 0,
            message: format!("🗑️  {} 개 항목을 삭제하시겠습니까?", selected_count),
            error: None,
        }
    }

    pub fn new_rename(current_name: String) -> Self {
        let cursor = current_name.len();
        DialogState {
            kind: DialogKind::Rename,
            input: current_name,
            cursor,
            message: "✏️  파일명 변경".to_string(),
            error: None,
        }
    }

    pub fn new_find() -> Self {
        DialogState {
            kind: DialogKind::Find,
            input: String::new(),
            cursor: 0,
            message: "🔍 파일 찾기 (현재 폴더 이하)".to_string(),
            error: None,
        }
    }

    pub fn new_batch_rename(selected_count: usize) -> Self {
        DialogState {
            kind: DialogKind::BatchRename,
            input: String::new(),
            cursor: 0,
            message: format!("✏️  {} 개 파일 배치 리네이밍", selected_count),
            error: None,
        }
    }

    pub fn new_ai_command() -> Self {
        DialogState {
            kind: DialogKind::AiCommand,
            input: String::new(),
            cursor: 0,
            message: "🤖 AI Commander - 파일 작업 명령 입력".to_string(),
            error: None,
        }
    }

    pub fn new_add_note(path: &str, existing_note: Option<String>) -> Self {
        let input = existing_note.unwrap_or_default();
        let cursor = input.len();
        DialogState {
            kind: DialogKind::AddNote,
            input,
            cursor,
            message: format!("📝 파일 메모/태그: {}", path),
            error: None,
        }
    }

    pub fn new_generate_script(selected_count: usize) -> Self {
        DialogState {
            kind: DialogKind::GenerateScript,
            input: String::new(),
            cursor: 0,
            message: format!("📜 {} 개 파일 배치 작업 스크립트 생성", selected_count),
            error: None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor <= self.input.len() {
            self.input.insert(self.cursor, c);
            self.cursor += c.len_utf8();
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            // 커서 위치 앞의 문자를 찾아서 제거
            let ch = self.input[..self.cursor].chars().last().unwrap();
            self.cursor -= ch.len_utf8();
            self.input.remove(self.cursor);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            // 한 문자 뒤로 이동
            let ch = self.input[..self.cursor].chars().last().unwrap();
            self.cursor -= ch.len_utf8();
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            // 한 문자 앞으로 이동
            let ch = self.input[self.cursor..].chars().next().unwrap();
            self.cursor += ch.len_utf8();
        }
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

pub fn render_dialog(frame: &mut Frame, area: Rect, dialog: &DialogState) {
    // 먼저 전체 영역을 검은색으로 덮기
    frame.render_widget(Clear, area);
    let overlay_style = Style::default().bg(Color::Black);
    let overlay = Block::default().style(overlay_style);
    frame.render_widget(overlay, area);

    // 다이얼로그 크기 계산
    let dialog_width = (area.width as f32 * 0.68) as u16;
    let dialog_height = match dialog.kind {
        DialogKind::Delete => 10,
        _ => 12,
    };
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_rect = Rect {
        x,
        y,
        width: dialog_width.max(42),
        height: dialog_height,
    };

    // 타이틀 및 색상 결정
    let (title, border_color) = match dialog.kind {
        DialogKind::Copy => ("  📋 파일 복사  ", Color::Cyan),
        DialogKind::Move => ("  ➡️  파일 이동  ", Color::Cyan),
        DialogKind::Mkdir => ("  📁 폴더 생성  ", Color::Green),
        DialogKind::Delete => ("  🗑️  파일 삭제  ", Color::Red),
        DialogKind::Rename => ("  ✏️  파일명 변경  ", Color::Cyan),
        DialogKind::Find => ("  🔍 파일 찾기  ", Color::Magenta),
        DialogKind::Pack => ("  📦 압축 파일 생성  ", Color::Yellow),
        DialogKind::AiCommand => ("  🤖 AI 명령 실행  ", Color::Magenta),
        DialogKind::BatchRename => ("  ✏️  배치 리네이밍  ", Color::Cyan),
        DialogKind::AddNote => ("  📝 파일 메모/태그  ", Color::Yellow),
        DialogKind::GenerateScript => ("  📜 배치 스크립트 생성  ", Color::Green),
        };


    // 메인 박스
    let bg = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).bold())
        .title(title)
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(15, 15, 20)));

    frame.render_widget(bg.clone(), dialog_rect);

    let inner = bg.inner(dialog_rect);

    // 섹션 1: 메시지 (위에서 1칸 패딩)
    let msg_rect = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: 1,
    };
    let msg_style = Style::default()
        .fg(Color::White)
        .bold();
    frame.render_widget(Paragraph::new(dialog.message.as_str()).style(msg_style), msg_rect);

    // 섹션 구분선
    let sep_rect = Rect {
        x: inner.x + 1,
        y: inner.y + 2,
        width: inner.width.saturating_sub(2),
        height: 1,
    };
    frame.render_widget(Paragraph::new("─".repeat(sep_rect.width as usize)).style(Style::default().fg(Color::DarkGray)), sep_rect);

    match dialog.kind {
        DialogKind::Delete => {
            // Delete 다이얼로그: 경고 메시지
            let warn_rect = Rect {
                x: inner.x + 1,
                y: inner.y + 3,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let warn_text = "⚠️  이 작업은 되돌릴 수 없습니다!";
            frame.render_widget(
                Paragraph::new(warn_text)
                    .style(Style::default().fg(Color::Red).bold())
                    .alignment(Alignment::Center),
                warn_rect,
            );

            // 확인 메시지
            let confirm_rect = Rect {
                x: inner.x + 1,
                y: inner.y + 4,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let confirm_text = "정말로 삭제하시겠습니까?";
            frame.render_widget(
                Paragraph::new(confirm_text)
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Center),
                confirm_rect,
            );

            // 버튼 영역
            let btn_rect = Rect {
                x: inner.x + 1,
                y: inner.y + 6,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let buttons = vec![
                ratatui::text::Span::styled("[ Y ", Style::default().fg(Color::Red).bold()),
                ratatui::text::Span::styled("삭제 ", Style::default().fg(Color::Red)),
                ratatui::text::Span::styled("]   ", Style::default().fg(Color::Red).bold()),
                ratatui::text::Span::styled("[ N ", Style::default().fg(Color::Green).bold()),
                ratatui::text::Span::styled("취소 ", Style::default().fg(Color::Green)),
                ratatui::text::Span::styled("]", Style::default().fg(Color::Green).bold()),
            ];
            let btn_line = ratatui::text::Line::from(buttons);
            frame.render_widget(Paragraph::new(btn_line).alignment(Alignment::Center), btn_rect);
        }
        _ => {
            // 입력 필드 레이블
            let label_rect = Rect {
                x: inner.x + 1,
                y: inner.y + 3,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let label_text = match dialog.kind {
                DialogKind::Copy | DialogKind::Move => "대상 경로:",
                DialogKind::Mkdir => "디렉토리 이름:",
                DialogKind::Rename => "새 이름:",
                DialogKind::AiCommand => "작업 지시 (예: log 파일 삭제, 사진 images 폴더로 이동):",
                DialogKind::BatchRename => "이름 변경 패턴 (예: 날짜순 정렬, 접두사 추가):",
                DialogKind::AddNote => "메모 및 태그 (#태그 형식 권장):",
                DialogKind::GenerateScript => "스크립트 지시사항 (예: 모두 WebP로 변환, PDF 합치기):",
                _ => "",
            };
            frame.render_widget(
                Paragraph::new(label_text).style(Style::default().fg(Color::Gray)),
                label_rect,
            );

            // 입력 필드 배경 (테두리)
            let input_outer_rect = Rect {
                x: inner.x + 1,
                y: inner.y + 4,
                width: inner.width.saturating_sub(2),
                height: 2,
            };
            let input_box = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .style(Style::default().bg(Color::Rgb(5, 5, 10)));
            frame.render_widget(input_box, input_outer_rect);

            // 입력 텍스트 렌더링
            let input_rect = Rect {
                x: input_outer_rect.x + 1,
                y: input_outer_rect.y,
                width: input_outer_rect.width.saturating_sub(2),
                height: input_outer_rect.height,
            };

            let display_input = if dialog.cursor >= dialog.input.len() {
                dialog.input.clone() + " "
            } else {
                dialog.input.clone()
            };

            let mut spans = Vec::new();
            for (i, c) in display_input.chars().enumerate() {
                if i == dialog.cursor {
                    // 커서 위치는 밝은 배경으로
                    spans.push(ratatui::text::Span::styled(
                        c.to_string(),
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                    ));
                } else {
                    // 일반 텍스트는 흰색
                    spans.push(ratatui::text::Span::styled(
                        c.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }
            }
            let input_line = ratatui::text::Line::from(spans);
            frame.render_widget(Paragraph::new(input_line), input_rect);

            // 오류 메시지
            if let Some(error) = &dialog.error {
                let error_rect = Rect {
                    x: inner.x + 1,
                    y: inner.y + 6,
                    width: inner.width.saturating_sub(2),
                    height: 1,
                };
                let error_para = Paragraph::new(format!("❌ {}", error))
                    .style(Style::default().fg(Color::Red).bold());
                frame.render_widget(error_para, error_rect);
            }

            // 힌트 텍스트
            let hint_y = if dialog.error.is_some() { 8 } else { 7 };
            let hint_rect = Rect {
                x: inner.x + 1,
                y: inner.y + hint_y,
                width: inner.width.saturating_sub(2),
                height: 2,
            };

            let hint_line1 = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("◀", Style::default().fg(Color::Gray).bold()),
                ratatui::text::Span::raw("/"),
                ratatui::text::Span::styled("▶", Style::default().fg(Color::Gray).bold()),
                ratatui::text::Span::raw(" 이동   "),
                ratatui::text::Span::styled("Backspace", Style::default().fg(Color::Yellow).bold()),
                ratatui::text::Span::raw(" 삭제"),
            ]);

            let hint_line2 = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("ENTER", Style::default().fg(Color::Green).bold()),
                ratatui::text::Span::raw(" 확인   "),
                ratatui::text::Span::styled("ESC", Style::default().fg(Color::Red).bold()),
                ratatui::text::Span::raw(" 취소"),
            ]);

            frame.render_widget(
                Paragraph::new(vec![hint_line1, hint_line2])
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White)),
                hint_rect,
            );
        }
    }
}
