use crate::commands::Command;
use crate::config::Config;
use crate::events::{handle_event, handle_dialog_event, handle_search_event};
use crate::fs::create_local_fs;
use crate::panel::PanelState;
use crate::ui::dialog::DialogState;
use crate::ui;
use anyhow::Result;
use crossterm::event;
use ratatui::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use notify::{Watcher, RecursiveMode, RecommendedWatcher};
use std::io::{Read, Seek};

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Dialog,
    Search,
    Filter,
    Viewer,
}

use tokio::sync::mpsc;
use crate::ui::viewer::ViewerState;

pub struct App {
    pub left_panel: PanelState,
    pub right_panel: PanelState,
    pub active_panel: bool, // true = left, false = right
    pub mode: AppMode,
    pub dialog: Option<DialogState>,
    pub viewer: Option<ViewerState>,
    pub search_query: String,
    pub config: Config,
    pub should_quit: bool,
    pub tx: mpsc::UnboundedSender<Command>,
    pub rx: mpsc::UnboundedReceiver<Command>,
    pub watcher: Option<RecommendedWatcher>,
}

impl App {
    pub fn new() -> Result<Self> {
        let fs = Arc::new(create_local_fs());
        let config = Config::load().unwrap_or_default();
        let show_hidden = config.ui.show_hidden;

        let mut left_panel = PanelState::new(fs.clone())?;
        let mut right_panel = PanelState::new(fs)?;
        
        left_panel.set_show_hidden(show_hidden)?;
        right_panel.set_show_hidden(show_hidden)?;

        let (tx, rx) = mpsc::unbounded_channel();

        Ok(App {
            left_panel,
            right_panel,
            active_panel: true,
            mode: AppMode::Normal,
            dialog: None,
            viewer: None,
            search_query: String::new(),
            config,
            should_quit: false,
            tx,
            rx,
            watcher: None,
        })
    }

    pub async fn run<B: Backend>(mut self, mut terminal: Terminal<B>) -> Result<()> {
        // 초기 크기 계산 시작
        self.calculate_recursive_size(true);
        self.calculate_recursive_size(false);

        loop {
            terminal.draw(|frame| {
                ui::render(frame, &self);
            })?;

            if self.should_quit {
                break;
            }

            // 백그라운드 명령어 처리
            while let Ok(command) = self.rx.try_recv() {
                self.handle_command(command)?;
            }

            if event::poll(Duration::from_millis(50))? {
                let event = event::read()?;
                let command = match self.mode {
                    AppMode::Dialog => handle_dialog_event(event, &mut self.dialog),
                    AppMode::Search | AppMode::Filter => handle_search_event(event),
                    AppMode::Viewer => self.handle_viewer_event(event),
                    _ => handle_event(event),
                };
                self.handle_command(command)?;
            }
        }

        Ok(())
    }

    fn calculate_recursive_size(&mut self, is_left: bool) {
        let panel = if is_left { &mut self.left_panel } else { &mut self.right_panel };
        
        // 이미 계산 중이거나 아카이브인 경우 건너뜀 (아카이브는 나중에 지원)
        if panel.is_calculating || panel.archive_base.is_some() {
            return;
        }

        panel.is_calculating = true;
        let path = panel.path.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut total_size = 0;
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok()) {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
                // 부하를 줄이기 위해 가끔씩 양보
                tokio::task::yield_now().await;
            }
            let _ = tx.send(Command::UpdateTotalSize(is_left, total_size));
        });
    }

    fn start_tail_watcher(&mut self, path: std::path::PathBuf) -> Result<()> {
        let tx = self.tx.clone();
        let path_clone = path.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    let _ = tx.send(Command::ViewerTailUpdate(path_clone.to_string_lossy().to_string()));
                }
            }
        })?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);
        Ok(())
    }

    fn handle_viewer_event(&self, event: event::Event) -> Command {
        if let event::Event::Key(key) = event {
            match key.code {
                event::KeyCode::Esc | event::KeyCode::Char('q') => Command::CancelDialog,
                event::KeyCode::Char('w') if key.modifiers.contains(event::KeyModifiers::ALT) => Command::ToggleWrap,
                event::KeyCode::Up => Command::CursorUp,
                event::KeyCode::Down => Command::CursorDown,
                event::KeyCode::PageUp => Command::PageUp,
                event::KeyCode::PageDown => Command::PageDown,
                _ => Command::None,
            }
        } else {
            Command::None
        }
    }

    pub fn handle_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Quit => self.should_quit = true,
            Command::SwitchPanel => {
                if !matches!(self.mode, AppMode::Viewer) {
                    self.active_panel = !self.active_panel;
                }
            }
            Command::CursorUp => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.scroll_up();
                } else {
                    let panel = self.active_panel_mut();
                    panel.cursor_up();
                }
            }
            Command::CursorDown => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.scroll_down();
                } else {
                    let panel = self.active_panel_mut();
                    panel.cursor_down();
                }
            }
            Command::PageUp => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.page_up(20);
                } else {
                    let panel = self.active_panel_mut();
                    panel.page_up(10);
                }
            }
            Command::PageDown => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.page_down(20);
                } else {
                    let panel = self.active_panel_mut();
                    panel.page_down(10);
                }
            }
            Command::Navigate => {
                let is_left = self.active_panel;
                let panel = self.active_panel_mut();
                if let Some(entry) = panel.get_current_entry() {
                    let path = entry.path.clone();
                    let is_dir = entry.is_dir;
                    if is_dir {
                        panel.navigate_to(path)?;
                        self.calculate_recursive_size(is_left);
                    } else if is_archive_file(&path) {
                        match crate::fs::create_archive_fs(path.clone()) {
                            Ok(archive_fs) => {
                                panel.set_fs(std::sync::Arc::new(archive_fs), path, true)?;
                            }
                            Err(e) => {
                                // TODO: Show error in UI
                                tracing::error!("Failed to open archive: {}", e);
                            }
                        }
                    }
                }
            }
            Command::GoParent => {
                let is_left = self.active_panel;
                let panel = self.active_panel_mut();
                panel.go_parent()?;
                self.calculate_recursive_size(is_left);
            }
            Command::ToggleSelect => {
                let panel = self.active_panel_mut();
                panel.toggle_select();
            }
            Command::SelectAll => {
                let panel = self.active_panel_mut();
                panel.select_all();
            }
            Command::ClearSelection => {
                let panel = self.active_panel_mut();
                panel.clear_selection();
            }
            Command::Refresh => {
                self.left_panel.refresh()?;
                self.right_panel.refresh()?;
                self.calculate_recursive_size(true);
                self.calculate_recursive_size(false);
            }
            Command::ToggleHidden => {
                self.config.ui.show_hidden = !self.config.ui.show_hidden;
                let show_hidden = self.config.ui.show_hidden;
                self.left_panel.set_show_hidden(show_hidden)?;
                self.right_panel.set_show_hidden(show_hidden)?;
                let _ = self.config.save();
            }
            Command::Copy => {
                let selected_count = self.active_panel().selected.len().max(1);
                let dst_path = self.inactive_panel().path.display().to_string();
                self.dialog = Some(DialogState::new_copy(selected_count, dst_path));
                self.mode = AppMode::Dialog;
            }
            Command::Pack => {
                let selected_count = self.active_panel().selected.len().max(1);
                let default_name = "archive.zip".to_string();
                self.dialog = Some(DialogState::new_pack(selected_count, default_name));
                self.mode = AppMode::Dialog;
            }
            Command::Move => {
                let selected_count = self.active_panel().selected.len().max(1);
                let dst_path = self.inactive_panel().path.display().to_string();
                self.dialog = Some(DialogState::new_move(selected_count, dst_path));
                self.mode = AppMode::Dialog;
            }
            Command::Mkdir => {
                self.dialog = Some(DialogState::new_mkdir());
                self.mode = AppMode::Dialog;
            }
            Command::Delete => {
                let selected_count = self.active_panel().selected.len().max(1);
                self.dialog = Some(DialogState::new_delete(selected_count));
                self.mode = AppMode::Dialog;
            }
            Command::Rename => {
                let current_name = if let Some(entry) = self.active_panel().get_current_entry() {
                    entry.name.clone()
                } else {
                    String::new()
                };
                self.dialog = Some(DialogState::new_rename(current_name));
                self.mode = AppMode::Dialog;
            }
            Command::View => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if !entry.is_dir {
                        // Use 80% of screen width as wrap target, and actual height
                        let width = 100; 
                        let height = 40; // Fallback
                        match ViewerState::new(entry.path.clone(), width, height) {
                            Ok(vs) => {
                                let is_log = vs.format == crate::ui::viewer::FileFormat::Log;
                                let path = vs.path.clone();
                                self.viewer = Some(vs);
                                self.mode = AppMode::Viewer;
                                
                                if is_log {
                                    let _ = self.start_tail_watcher(path);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to open viewer: {}", e);
                            }
                        }
                    }
                }
            }
            Command::ToggleWrap => {
                if let Some(viewer) = &mut self.viewer {
                    let _ = viewer.toggle_wrap(100);
                }
            }
            Command::ConfirmDialog => {
                self.execute_dialog_operation()?;
                self.dialog = None;
                self.mode = AppMode::Normal;
                self.left_panel.refresh()?;
                self.right_panel.refresh()?;
            }
            Command::CancelDialog => {
                self.dialog = None;
                self.viewer = None;
                self.watcher = None; // Stop watching
                self.mode = AppMode::Normal;
            }
            Command::DialogInput(_) | Command::DialogBackspace | Command::DialogLeft | Command::DialogRight => {
                // Handled by handle_dialog_event, just ignore here
            }
            Command::QuickSearch('/') => {
                self.mode = AppMode::Search;
                self.search_query.clear();
                self.active_panel_mut().clear_filter();
            }
            Command::QuickSearch(c) => {
                self.mode = AppMode::Search;
                self.search_query.clear();
                self.search_query.push(c);
                let query = self.search_query.clone();
                self.active_panel_mut().apply_quick_filter(&query);
            }
            Command::Filter => {
                self.mode = AppMode::Filter;
                self.search_query.clear();
            }
            Command::Find => {
                self.dialog = Some(DialogState::new_find());
                self.mode = AppMode::Dialog;
            }
            Command::SearchInput(c) => {
                self.search_query.push(c);
                if matches!(self.mode, AppMode::Search) {
                    let query = self.search_query.clone();
                    self.active_panel_mut().apply_quick_filter(&query);
                }
            }
            Command::SearchBackspace => {
                self.search_query.pop();
                if matches!(self.mode, AppMode::Search) {
                    if self.search_query.is_empty() {
                        self.active_panel_mut().clear_filter();
                    } else {
                        let query = self.search_query.clone();
                        self.active_panel_mut().apply_quick_filter(&query);
                    }
                }
            }
            Command::SearchConfirm => {
                if matches!(self.mode, AppMode::Filter) {
                    let query = self.search_query.clone();
                    self.active_panel_mut().apply_wildcard_filter(&query);
                }
                self.mode = AppMode::Normal;
            }
            Command::SearchCancel => {
                self.active_panel_mut().clear_filter();
                self.search_query.clear();
                self.mode = AppMode::Normal;
            }
            Command::UpdateTotalSize(is_left, size) => {
                let panel = if is_left { &mut self.left_panel } else { &mut self.right_panel };
                panel.recursive_total_size = Some(size);
                panel.is_calculating = false;
            }
            Command::ViewerTailUpdate(_) => {
                if let Some(viewer) = &mut self.viewer {
                    if viewer.is_tail_active {
                        if let Ok(mut file) = std::fs::File::open(&viewer.path) {
                            if let Ok(metadata) = file.metadata() {
                                let new_len = metadata.len();
                                if new_len > viewer.last_offset {
                                    let mut new_content = String::new();
                                    if file.seek(std::io::SeekFrom::Start(viewer.last_offset)).is_ok() {
                                        if file.read_to_string(&mut new_content).is_ok() {
                                            viewer.append_new_content(&new_content, 100);
                                            viewer.last_offset = new_len;
                                        }
                                    }
                                } else if new_len < viewer.last_offset {
                                    // File truncated?
                                    viewer.last_offset = 0;
                                }
                            }
                        }
                    }
                }
            }
            _ => {} // Other commands will be implemented in later phases
        }
        Ok(())
    }

    pub fn active_panel(&self) -> &PanelState {
        if self.active_panel {
            &self.left_panel
        } else {
            &self.right_panel
        }
    }

    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        if self.active_panel {
            &mut self.left_panel
        } else {
            &mut self.right_panel
        }
    }

    fn inactive_panel(&self) -> &PanelState {
        if self.active_panel {
            &self.right_panel
        } else {
            &self.left_panel
        }
    }

    pub fn execute_dialog_operation(&mut self) -> Result<()> {
        use crate::ui::dialog::DialogKind;
        use std::path::PathBuf;

        let dialog = match self.dialog.take() {
            Some(d) => d,
            None => return Ok(()),
        };

        match dialog.kind {
            DialogKind::Find => {
                let query = dialog.input.clone();
                let root = self.active_panel().path.clone();
                let results = crate::ops::search::find_files(&root, &query);
                self.active_panel_mut().set_find_results(results);
            }
            DialogKind::Copy => {
                let dst_dir = PathBuf::from(&dialog.input);
                let panel = self.active_panel_mut();
                let mut srcs: Vec<_> = panel.get_selected_entries().iter().map(|e| e.path.clone()).collect();

                if srcs.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        srcs.push(entry.path.clone());
                    }
                }

                for src in srcs {
                    let file_name = src.file_name().unwrap_or_default();
                    let dst = dst_dir.join(file_name);
                    panel.fs.copy(&src, &dst, true)?;
                }
            }
            DialogKind::Move => {
                let dst_dir = PathBuf::from(&dialog.input);
                let panel = self.active_panel_mut();
                let mut srcs: Vec<_> = panel.get_selected_entries().iter().map(|e| e.path.clone()).collect();

                if srcs.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        srcs.push(entry.path.clone());
                    }
                }

                for src in srcs {
                    let file_name = src.file_name().unwrap_or_default();
                    let dst = dst_dir.join(file_name);
                    panel.fs.move_entry(&src, &dst)?;
                }
            }
            DialogKind::Mkdir => {
                let dir_name = dialog.input.clone();
                let panel = self.active_panel_mut();
                panel.fs.mkdir(&panel.path.join(&dir_name))?;
            }
            DialogKind::Delete => {
                let panel = self.active_panel_mut();
                let mut srcs: Vec<_> = panel.get_selected_entries().iter().map(|e| e.path.clone()).collect();

                if srcs.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        srcs.push(entry.path.clone());
                    }
                }

                for src in srcs {
                    if src.is_dir() {
                        panel.fs.delete(&src, true)?;
                    } else {
                        panel.fs.delete(&src, false)?;
                    }
                }
            }
            DialogKind::Rename => {
                let new_name = dialog.input.clone();
                let panel = self.active_panel_mut();
                if let Some(entry) = panel.get_current_entry() {
                    panel.fs.rename(&entry.path, &new_name)?;
                }
            }
            DialogKind::Pack => {
                let archive_name = dialog.input.clone();
                let dst_path = self.inactive_panel().path.join(archive_name);
                let panel = self.active_panel_mut();
                let mut srcs: Vec<_> = panel.get_selected_entries().iter().map(|e| e.path.clone()).collect();

                if srcs.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        srcs.push(entry.path.clone());
                    }
                }

                crate::ops::archive::pack_files(&srcs, &dst_path)?;
            }
        }
        Ok(())
    }
}

fn is_archive_file(path: &std::path::Path) -> bool {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(extension.as_str(), "zip" | "tar" | "gz" | "tgz")
}
