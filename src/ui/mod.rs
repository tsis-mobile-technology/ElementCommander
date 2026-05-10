mod cmdbar;
mod panel;
mod statusbar;
mod searchbar;
pub mod dialog;
pub mod viewer;
pub mod ai;
pub mod ai_command;

use crate::app::{App, AppMode};
use ratatui::prelude::*;
use ratatui::layout::Constraint;
use ratatui::widgets::{List, ListItem, Block, Borders};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    if matches!(app.mode, AppMode::Viewer) {
        if let Some(viewer) = &app.viewer {
            render_viewer_mode(frame, size, app, viewer);
            return;
        }
    }

    if matches!(app.mode, AppMode::AiChat) {
        if let Some(ai_state) = &app.ai_state {
            ai::render_ai_mode(frame, ai_state);
            return;
        }
    }

    if matches!(app.mode, AppMode::AiCommandConfirm) {
        if let Some(command_state) = &app.ai_command_state {
            ai_command::render_ai_command_confirm(frame, command_state);
            return;
        }
    }

    // Determine if search bar should be shown
    let show_search = matches!(app.mode, AppMode::Search | AppMode::Filter);

    // Main layout: title + content + [search] + cmdbar + statusbar
    let mut constraints = vec![
        Constraint::Length(1), // Title
        Constraint::Min(0),     // Content (panels)
    ];
    
    if show_search {
        constraints.push(Constraint::Length(1)); // Search bar
    }
    
    constraints.push(Constraint::Length(1)); // Command bar
    constraints.push(Constraint::Length(1)); // Status bar

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    // Title
    let title = Line::from(" 🗂️  hermes_tail - Total Commander TUI File Manager".bold());
    frame.render_widget(title, chunks[0]);

    // Dual panel layout
    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Render panels
    panel::render_panel(frame, panel_chunks[0], &app.left_panel, app.active_panel);
    panel::render_panel(frame, panel_chunks[1], &app.right_panel, !app.active_panel);

    let mut next_chunk_idx = 2;

    // Render search bar if active
    if show_search {
        searchbar::render_searchbar(frame, chunks[next_chunk_idx], app);
        next_chunk_idx += 1;
    }

    // Command bar
    cmdbar::render_cmdbar(frame, chunks[next_chunk_idx]);
    next_chunk_idx += 1;

    // Status bar
    statusbar::render_statusbar(frame, chunks[next_chunk_idx], &app.left_panel, &app.right_panel);

    // Dialog overlay (rendered last, on top)
    if let Some(dialog) = &app.dialog {
        dialog::render_dialog(frame, frame.area(), dialog);
    }
}

fn render_viewer_mode(frame: &mut Frame, area: Rect, _app: &App, viewer: &viewer::ViewerState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title/Path
            Constraint::Min(0),     // Content
            Constraint::Length(1), // Status/Keys
        ])
        .split(area);

    // Title bar for viewer
    let wrap_status = if viewer.wrap { " [WRAP]" } else { " [NO-WRAP]" };
    let tail_tag = if viewer.is_tail_active {
        Span::styled(" TAIL ", Style::default().bg(Color::Green).fg(Color::Black).bold())
    } else {
        Span::styled(" VIEW ", Style::default().bg(Color::Yellow).fg(Color::Black).bold())
    };

    let title = Line::from(vec![
        tail_tag,
        Span::raw(format!(" - {}{}", viewer.path.display(), wrap_status)),
    ]);
    frame.render_widget(title, chunks[0]);

    // Content rendering
    let mut items = Vec::new();
    let visible_height = chunks[1].height as usize;
    let start = viewer.scroll;
    let end = (start + visible_height).min(viewer.lines.len());

    for line in &viewer.lines[start..end] {
        if !line.styled.is_empty() {
            let spans: Vec<Span> = line.styled.iter().map(|(color, text, is_bold)| {
                let mut style = Style::default().fg(*color);
                if *is_bold {
                    style = style.bold();
                }
                Span::styled(text.clone(), style)
            }).collect();
            items.push(ListItem::new(Line::from(spans)));
        } else {
            items.push(ListItem::new(Line::from(line.raw.as_str())));
        }
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE).bg(Color::Black));
    frame.render_widget(list, chunks[1]);

    // Status bar for viewer
    let status = Line::from(vec![
        Span::styled(" Esc ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" Close  "),
        Span::styled(" Alt+W ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" Toggle Wrap  "),
        Span::styled(format!(" Line: {}/{} ", viewer.scroll + 1, viewer.lines.len()), Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(status, chunks[2]);
}
