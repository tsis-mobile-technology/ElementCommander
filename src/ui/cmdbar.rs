use ratatui::prelude::*;
use super::theme::Theme;

pub fn render_cmdbar(frame: &mut Frame, area: Rect, theme: &Theme) {
    let cmdbar = Line::from(vec![
        Span::styled("F1", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Help  "),
        Span::styled("F3", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":View  "),
        Span::styled("F5", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Copy  "),
        Span::styled("F6", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Move  "),
        Span::styled("F7", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Mkdir  "),
        Span::styled("F8", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Delete  "),
        Span::styled("Tab", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Switch  "),
        Span::styled("Q", Style::default().fg(theme.cmdbar_key_fg).bold()),
        Span::raw(":Quit  "),
        // AI 커멘더
        Span::styled("🤖", Style::default().fg(theme.ai_color).bold()),
        Span::raw(" "),
        Span::styled("C-G", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":NL  "),
        Span::styled("A-G", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Sum  "),
        Span::styled("A-S", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Sec  "),
        Span::styled("A-I", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Img  "),
        Span::styled("A-C", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Code  "),
        Span::styled("A-D", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Diff  "),
        Span::styled("A-A", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":Fold  "),
        Span::styled("A-R", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":RD  "),
        Span::styled("A-X", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":SC  "),
        Span::styled("A-B", Style::default().fg(theme.ai_color).bold()),
        Span::raw(":BR "),
    ])
    .style(Style::default().bg(theme.cmdbar_bg).fg(Color::White));

    frame.render_widget(cmdbar, area);
}
