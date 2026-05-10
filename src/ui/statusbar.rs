use crate::panel::PanelState;
use crate::fs::FileEntry;
use super::theme::Theme;
use ratatui::prelude::*;

pub fn render_statusbar(frame: &mut Frame, area: Rect, left: &PanelState, right: &PanelState, theme: &Theme) {
    let left_size_str = match left.recursive_total_size {
        Some(s) => format!("Total: {}", FileEntry::format_size(s)),
        None => if left.is_calculating {
            format!("Total: ... ({})", FileEntry::format_size(left.list_total_size))
        } else {
            format!("Total: {}", FileEntry::format_size(left.list_total_size))
        }
    };

    let right_size_str = match right.recursive_total_size {
        Some(s) => format!("Total: {}", FileEntry::format_size(s)),
        None => if right.is_calculating {
            format!("Total: ... ({})", FileEntry::format_size(right.list_total_size))
        } else {
            format!("Total: {}", FileEntry::format_size(right.list_total_size))
        }
    };

    let left_info = format!(
        "📁 {} | {} items | {} selected | {}",
        left.path.display(),
        left.entries.len(),
        left.selected.len(),
        left_size_str
    );

    let right_info = format!(
        "{} | {} selected | {} items | 📁 {}",
        right_size_str,
        right.selected.len(),
        right.entries.len(),
        right.path.display(),
    );

    let total_width = area.width as usize;
    let left_width = total_width / 2;
    let right_width = total_width - left_width;

    let left_padded = format!("{:<width$}", left_info, width = left_width);
    let right_padded = format!("{:>width$}", right_info, width = right_width);

    let statusbar = Line::from(vec![
        Span::styled(left_padded, Style::default().bg(theme.statusbar_bg)),
        Span::styled(right_padded, Style::default().bg(theme.statusbar_bg)),
    ]);

    frame.render_widget(statusbar, area);
}
