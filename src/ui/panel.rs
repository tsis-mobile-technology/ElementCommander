use crate::panel::PanelState;
use super::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub fn render_panel(frame: &mut Frame, area: Rect, panel: &PanelState, is_active: bool, theme: &Theme, notes: &crate::config::NotesStore) {
    // 활성 패널: 테마 배경색, 흰색 텍스트
    // 비활성 패널: 테마 비활성 배경색, 회색 텍스트
    let (border_style, _title_style, bg_color) = if is_active {
        (
            Style::default()
                .fg(theme.active_border)
                .bg(theme.active_bg)
                .bold(),
            Style::default()
                .fg(theme.active_cursor_fg)
                .bg(theme.active_cursor_bg)
                .bold(),
            theme.active_bg,
        )
    } else {
        (
            Style::default()
                .fg(theme.inactive_border)
                .bg(theme.inactive_bg),
            Style::default()
                .fg(Color::Gray)
                .bg(theme.inactive_bg),
            theme.inactive_bg,
        )
    };

    // 패널 제목 (활성 패널은 노란 배경)
    let title_text = if is_active {
        format!(" ▶ {} ◀ ", panel.path.display())
    } else {
        format!(" {} ", panel.path.display())
    };

    let block = Block::default()
        .title(title_text)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style);

    // 패널 너비 계산 (테두리 제외)
    let panel_width = area.width.saturating_sub(2) as usize;

    // 고정 컬럼 너비
    let marker_width = 4;        // "[✓] " or "[>] " or "   "
    let icon_width = 2;          // "📁" or "📄"
    let size_width = 10;         // "    123.4K" 정렬
    let date_width = 17;         // "YYYY-MM-DD HH:MM"
    let padding = 2;             // 사이 공백

    // 파일명에 할당할 너비 (동적)
    let name_width = panel_width.saturating_sub(
        marker_width + icon_width + size_width + date_width + (padding * 3)
    );

    let visible = panel.visible_entries();
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = panel.selected.contains(&idx);
            let is_current = idx == panel.cursor;

            let marker = if is_selected {
                "[✓]"
            } else if is_current {
                "[>]"
            } else {
                "   "
            };

            let icon = if entry.is_dir { "📁" } else { "📄" };

            // 메모 여부 확인
            let note_indicator = if notes.get_note(&entry.path.to_string_lossy()).is_some() {
                " [N]"
            } else {
                ""
            };

            // 파일명 길이 제한 (너무 길면 생략 부호)
            // UTF-8 안전한 문자 단위 자르기
            let mut display_name = if entry.name.chars().count() > name_width.saturating_sub(note_indicator.chars().count()) {
                entry.name
                    .chars()
                    .take(name_width.saturating_sub(1 + note_indicator.chars().count()))
                    .collect::<String>()
                    + "…"
            } else {
                entry.name.clone()
            };
            display_name.push_str(note_indicator);

            // 동적 너비로 텍스트 포맷
            let text = format!(
                "{} {} {:<width$}  {:>size$}  {}",
                marker,
                icon,
                display_name,
                entry.display_size(),
                entry.display_modified(),
                width = name_width,
                size = size_width - 2
            );

            // 현재 항목: 테마 커서 색상
            // 선택된 항목: 테마 선택 색상
            // 일반 항목: 배경색 상속
            let style = if is_current {
                if is_active {
                    Style::default()
                        .fg(theme.active_cursor_fg)
                        .bg(theme.active_cursor_bg)
                        .bold()
                } else {
                    Style::default()
                        .fg(theme.inactive_cursor_fg)
                        .bg(theme.inactive_cursor_bg)
                        .bold()
                }
            } else if is_selected {
                Style::default()
                    .fg(theme.selected_fg)
                    .bg(bg_color)
                    .bold()
            } else {
                if is_active {
                    Style::default()
                        .fg(Color::White)
                        .bg(bg_color)
                } else {
                    Style::default()
                        .fg(Color::Gray)
                        .bg(bg_color)
                }
            };

            ListItem::new(text).style(style)
        })
        .collect();

    // 스크롤 상태 생성
    let mut list_state = ListState::default();
    list_state.select(Some(panel.cursor));

    let list = List::new(items)
        .block(block);

    // StatefulWidget을 사용하여 스크롤 가능하게 렌더링
    frame.render_stateful_widget(list, area, &mut list_state);
}
