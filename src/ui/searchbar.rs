use crate::app::{App, AppMode};
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render_searchbar(frame: &mut Frame, area: Rect, app: &App) {
    let (prefix, color) = match app.mode {
        AppMode::Search => (" QUICK SEARCH: ", Color::Yellow),
        AppMode::Filter => (" WILDCARD FILTER: ", Color::Green),
        _ => return,
    };

    // Render as a bar with a blinking cursor effect
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(prefix, Style::default().fg(color).bold()),
        Span::raw(&app.search_query),
        Span::styled("█", Style::default().fg(color).add_modifier(Modifier::SLOW_BLINK)),
    ])).bg(Color::Black), area);
}
