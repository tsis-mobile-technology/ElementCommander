use ratatui::prelude::*;

pub fn render_cmdbar(frame: &mut Frame, area: Rect) {
    let cmdbar = Line::from(vec![
        Span::styled("F3", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":View  "),
        Span::styled("F5", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Copy  "),
        Span::styled("F6", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Move  "),
        Span::styled("F7", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Mkdir  "),
        Span::styled("F8", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Delete  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Switch  "),
        Span::styled("Q", Style::default().fg(Color::Yellow).bold()),
        Span::raw(":Quit  "),
    ])
    .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(cmdbar, area);
}
