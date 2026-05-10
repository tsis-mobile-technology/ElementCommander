use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub active_bg: Color,
    pub active_border: Color,
    pub active_cursor_fg: Color,
    pub active_cursor_bg: Color,
    pub selected_fg: Color,
    pub inactive_bg: Color,
    pub inactive_border: Color,
    pub inactive_cursor_fg: Color,
    pub inactive_cursor_bg: Color,
    pub statusbar_bg: Color,
    pub cmdbar_bg: Color,
    pub cmdbar_key_fg: Color,
    pub ai_color: Color,
}

impl Theme {
    pub fn classic() -> Self {
        Self {
            active_bg: Color::Blue,
            active_border: Color::White,
            active_cursor_fg: Color::Black,
            active_cursor_bg: Color::Yellow,
            selected_fg: Color::Green,
            inactive_bg: Color::Black,
            inactive_border: Color::DarkGray,
            inactive_cursor_fg: Color::Yellow,
            inactive_cursor_bg: Color::DarkGray,
            statusbar_bg: Color::DarkGray,
            cmdbar_bg: Color::Blue,
            cmdbar_key_fg: Color::Yellow,
            ai_color: Color::Cyan,
        }
    }

    pub fn dark() -> Self {
        Self {
            active_bg: Color::Black,
            active_border: Color::Cyan,
            active_cursor_fg: Color::Black,
            active_cursor_bg: Color::Cyan,
            selected_fg: Color::Magenta,
            inactive_bg: Color::Black,
            inactive_border: Color::DarkGray,
            inactive_cursor_fg: Color::Cyan,
            inactive_cursor_bg: Color::DarkGray,
            statusbar_bg: Color::Rgb(30, 30, 30),
            cmdbar_bg: Color::Rgb(40, 40, 40),
            cmdbar_key_fg: Color::Cyan,
            ai_color: Color::Green,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self::dark(),
            _ => Self::classic(),
        }
    }
}
