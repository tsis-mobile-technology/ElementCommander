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
        Span::styled("C-G", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":NL  "),
        Span::styled("A-G", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Sum  "),
        Span::styled("A-S", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Sec  "),
        Span::styled("A-I", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Img  "),
        Span::styled("A-C", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Code  "),
        Span::styled("A-D", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Diff  "),
        Span::styled("A-A", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":Fold  "),
        Span::styled("A-F2", Style::default().fg(Color::Cyan).bold()),
        Span::raw(":BRen "),
    ])
    .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(cmdbar, area);
}
