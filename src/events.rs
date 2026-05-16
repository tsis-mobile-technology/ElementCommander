use crate::commands::Command;
use crate::ui::dialog::{DialogState, DialogKind};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn handle_event(event: Event) -> Command {
    match event {
        Event::Key(key) => handle_key_event(key),
        _ => Command::None,
    }
}

pub fn handle_key_event(key: KeyEvent) -> Command {
    match key.code {
        KeyCode::F(1) => Command::ShowHelp,
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Quit,
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::AiNaturalCommand,
        KeyCode::Tab => Command::SwitchPanel,
        KeyCode::Up => Command::CursorUp,
        KeyCode::Down => Command::CursorDown,
        KeyCode::PageUp => Command::PageUp,
        KeyCode::PageDown => Command::PageDown,
        KeyCode::Enter => Command::Navigate,
        KeyCode::Backspace => Command::GoParent,
        KeyCode::Insert => Command::ToggleSelect,
        KeyCode::Esc => Command::ClearSelection,
        KeyCode::F(2) if key.modifiers.contains(KeyModifiers::SHIFT) => Command::Rename,
        KeyCode::F(2) => Command::Rename,
        KeyCode::F(3) => Command::View,
        KeyCode::F(5) if key.modifiers.contains(KeyModifiers::ALT) => Command::Pack,
        KeyCode::F(5) => Command::Copy,
        KeyCode::F(6) if key.modifiers.contains(KeyModifiers::SHIFT) => Command::Rename,
        KeyCode::F(6) => Command::Move,
        KeyCode::F(7) => Command::Mkdir,
        KeyCode::F(8) => Command::Delete,
        KeyCode::F(10) => Command::Quit,
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::SelectAll,
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Find,
        // AI 커멘더 단축키
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiSummarize,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiSecurityScan,
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiImageInfo,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiCodeStructure,
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiFileDiff,
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiFolderAnalysis,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiFindDuplicates,
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiOldFiles,
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiGenerateReadme,
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiAddNote,
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::ALT) => Command::AiGenerateScript,
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => Command::BatchRename,
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::ToggleHidden,
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => Command::Refresh,
        KeyCode::Char('=') => Command::Filter,
        KeyCode::Char('/') => Command::QuickSearch('/'),
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => Command::QuickSearch(c),
        _ => Command::None,
    }
}

pub fn handle_dialog_event(event: Event, dialog: &mut Option<DialogState>) -> Command {
    match event {
        Event::Key(key) => {
            if let Some(dialog) = dialog {
                match key.code {
                    KeyCode::Esc => Command::CancelDialog,
                    KeyCode::Enter => {
                        // Delete dialog needs Y/N response
                        if matches!(dialog.kind, DialogKind::Delete) {
                            Command::None
                        } else {
                            Command::ConfirmDialog
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') if matches!(dialog.kind, DialogKind::Delete) => {
                        Command::ConfirmDialog
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') if matches!(dialog.kind, DialogKind::Delete) => {
                        Command::CancelDialog
                    }
                    KeyCode::Backspace => {
                        if !matches!(dialog.kind, DialogKind::Delete) {
                            dialog.backspace();
                            dialog.clear_error();
                        }
                        Command::None
                    }
                    KeyCode::Left => {
                        if !matches!(dialog.kind, DialogKind::Delete) {
                            dialog.cursor_left();
                        }
                        Command::None
                    }
                    KeyCode::Right => {
                        if !matches!(dialog.kind, DialogKind::Delete) {
                            dialog.cursor_right();
                        }
                        Command::None
                    }
                    KeyCode::Char(c) if !matches!(dialog.kind, DialogKind::Delete) => {
                        dialog.insert_char(c);
                        dialog.clear_error();
                        Command::None
                    }
                    _ => Command::None,
                }
            } else {
                Command::None
            }
        }
        _ => Command::None,
    }
}

pub fn handle_search_event(event: Event) -> Command {
    match event {
        Event::Key(key) => {
            match key.code {
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Command::SearchInput(c)
                }
                KeyCode::Backspace => Command::SearchBackspace,
                KeyCode::Esc => Command::SearchCancel,
                KeyCode::Enter => Command::SearchConfirm,
                KeyCode::Up => Command::CursorUp,
                KeyCode::Down => Command::CursorDown,
                KeyCode::PageUp => Command::PageUp,
                KeyCode::PageDown => Command::PageDown,
                _ => Command::None,
            }
        }
        _ => Command::None,
    }
}

pub fn handle_ai_event(event: Event) -> Command {
    match event {
        Event::Key(key) => {
            match key.code {
                KeyCode::Esc => Command::AiCancel,
                KeyCode::Up => Command::AiScrollUp,
                KeyCode::Down => Command::AiScrollDown,
                KeyCode::PageUp => Command::AiPageUp,
                KeyCode::PageDown => Command::AiPageDown,
                KeyCode::Char('q') => Command::AiCancel,
                KeyCode::Char('t') | KeyCode::Char('T') => Command::AiToggleThinking,
                _ => Command::None,
            }
        }
        _ => Command::None,
    }
}

pub fn handle_ai_command_confirm_event(event: Event) -> Command {
    match event {
        Event::Key(key) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Command::AiCommandConfirm,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Command::AiCommandCancel,
                KeyCode::Up => Command::AiCommandScrollUp,
                KeyCode::Down => Command::AiCommandScrollDown,
                _ => Command::None,
            }
        }
        _ => Command::None,
    }
}
