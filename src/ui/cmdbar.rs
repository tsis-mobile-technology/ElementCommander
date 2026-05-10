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
        // AI 커멘더
        Span::styled("🤖", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" "),
        Span::styled("G", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Summarize "),
        Span::styled("S", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Security "),
        Span::styled("I", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Image "),
        Span::styled("C", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Code "),
        Span::styled("D", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Diff "),
        Span::styled("A", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Folder "),
    ])
    .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(cmdbar, area);
}
