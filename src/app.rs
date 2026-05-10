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
    AiChat,
    AiCommandConfirm,
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
    pub ai_state: Option<crate::ai::AiState>,
    pub ai_command_state: Option<crate::ai::AiCommandState>,
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
            ai_state: None,
            ai_command_state: None,
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
                    AppMode::AiChat => crate::events::handle_ai_event(event),
                    AppMode::AiCommandConfirm => crate::events::handle_ai_command_confirm_event(event),
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
            Command::BatchRename => {
                let panel = self.active_panel();
                let mut files: Vec<_> = panel.get_selected_entries()
                    .iter()
                    .map(|e| e.path.clone())
                    .collect();
                if files.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        files.push(entry.path.clone());
                    }
                }
                if files.is_empty() {
                    return Ok(());
                }
                let selected_count = files.len();
                self.dialog = Some(crate::ui::dialog::DialogState::new_batch_rename(selected_count));
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
                let is_async_dialog = self.dialog.as_ref()
                    .map(|d| matches!(d.kind, crate::ui::dialog::DialogKind::AiCommand | crate::ui::dialog::DialogKind::BatchRename))
                    .unwrap_or(false);

                self.execute_dialog_operation()?;

                // AiCommand와 BatchRename은 execute_dialog_operation 내에서 mode를 이미 AiChat으로 설정했으므로
                // 여기서 mode를 Normal로 변경하면 안 됨
                if !is_async_dialog {
                    self.dialog = None;
                    self.mode = AppMode::Normal;
                    self.left_panel.refresh()?;
                    self.right_panel.refresh()?;
                }
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
            Command::AiSummarize => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if !entry.is_dir {
                        let path = entry.path.clone();
                        let file_path = path.display().to_string();
                        tracing::info!("AI 요약 시작: {}", file_path);
                        self.ai_state = Some(crate::ai::AiState::loading(file_path.clone()));
                        self.mode = AppMode::AiChat;

                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            let client = crate::ai::AiClient::new(
                                "http://localhost:8080/v1".to_string(),
                                "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                            );

                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    if content.len() > 50000 {
                                        let truncated = format!("{}... (파일이 너무 커서 처음 50KB만 표시)", &content[..50000]);
                                        match client.summarize_file(truncated, file_path).await {
                                            Ok(ai_response) => {
                                                let _ = tx.send(Command::AiResponse(ai_response));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Command::AiError(e.to_string()));
                                            }
                                        }
                                    } else {
                                        match client.summarize_file(content, file_path).await {
                                            Ok(ai_response) => {
                                                let _ = tx.send(Command::AiResponse(ai_response));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Command::AiError(e.to_string()));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Command::AiError(format!("파일을 읽을 수 없습니다: {}", e)));
                                }
                            }
                        });
                    }
                }
            }

            Command::AiSecurityScan => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if !entry.is_dir {
                        let path = entry.path.clone();
                        let file_path = path.display().to_string();
                        tracing::info!("보안 스캔 시작: {}", file_path);
                        self.ai_state = Some(crate::ai::AiState::loading(format!("보안 스캔: {}", file_path)));
                        self.mode = AppMode::AiChat;

                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            let client = crate::ai::AiClient::new(
                                "http://localhost:8080/v1".to_string(),
                                "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                            );

                            match std::fs::read_to_string(&path) {
                                Ok(content) => {
                                    let truncated = if content.len() > 50000 {
                                        format!("{}... (처음 50KB만 스캔)", &content[..50000])
                                    } else {
                                        content
                                    };
                                    match client.scan_security(truncated, file_path).await {
                                        Ok(ai_response) => {
                                            let _ = tx.send(Command::AiResponse(ai_response));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Command::AiError(e.to_string()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Command::AiError(format!("파일을 읽을 수 없습니다: {}", e)));
                                }
                            }
                        });
                    }
                }
            }

            Command::AiImageInfo => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if !entry.is_dir {
                        let path = entry.path.clone();
                        let file_path = path.display().to_string();
                        let is_image = matches!(
                            path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff"
                        );

                        if is_image {
                            tracing::info!("이미지 분석 시작: {}", file_path);
                            self.ai_state = Some(crate::ai::AiState::loading(format!("이미지 분석: {}", file_path)));
                            self.mode = AppMode::AiChat;

                            let tx = self.tx.clone();
                            tokio::spawn(async move {
                                let client = crate::ai::AiClient::new(
                                    "http://localhost:8080/v1".to_string(),
                                    "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                                );

                                // 이미지 파일 정보 (간단한 메타 정보 수집)
                                match std::fs::metadata(&path) {
                                    Ok(metadata) => {
                                        let file_info = format!(
                                            "파일명: {}\n크기: {} bytes\n수정일: {:?}",
                                            path.file_name().unwrap_or_default().to_string_lossy(),
                                            metadata.len(),
                                            metadata.modified()
                                        );
                                        match client.analyze_image(file_info, file_path).await {
                                            Ok(ai_response) => {
                                                let _ = tx.send(Command::AiResponse(ai_response));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Command::AiError(e.to_string()));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Command::AiError(format!("파일 정보를 읽을 수 없습니다: {}", e)));
                                    }
                                }
                            });
                        } else {
                            self.ai_state = Some(crate::ai::AiState::error("이미지 파일을 선택해주세요".to_string()));
                            self.mode = AppMode::AiChat;
                        }
                    }
                }
            }

            Command::AiCodeStructure => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if !entry.is_dir {
                        let path = entry.path.clone();
                        let file_path = path.display().to_string();
                        let is_code = matches!(
                            path.extension().and_then(|s| s.to_str()).unwrap_or(""),
                            "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" | "rb" | "php"
                        );

                        if is_code {
                            tracing::info!("코드 분석 시작: {}", file_path);
                            self.ai_state = Some(crate::ai::AiState::loading(format!("코드 분석: {}", file_path)));
                            self.mode = AppMode::AiChat;

                            let tx = self.tx.clone();
                            tokio::spawn(async move {
                                let client = crate::ai::AiClient::new(
                                    "http://localhost:8080/v1".to_string(),
                                    "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                                );

                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let truncated = if content.len() > 50000 {
                                            format!("{}... (처음 50KB만 분석)", &content[..50000])
                                        } else {
                                            content
                                        };
                                        match client.analyze_code(truncated, file_path).await {
                                            Ok(ai_response) => {
                                                let _ = tx.send(Command::AiResponse(ai_response));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(Command::AiError(e.to_string()));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Command::AiError(format!("파일을 읽을 수 없습니다: {}", e)));
                                    }
                                }
                            });
                        } else {
                            self.ai_state = Some(crate::ai::AiState::error("코드 파일을 선택해주세요".to_string()));
                            self.mode = AppMode::AiChat;
                        }
                    }
                }
            }

            Command::AiFileDiff => {
                if !self.active_panel().selected.is_empty() && self.active_panel().selected.len() == 2 {
                    let entries = self.active_panel().get_selected_entries();
                    if entries.len() == 2 && !entries[0].is_dir && !entries[1].is_dir {
                        let path1 = entries[0].path.clone();
                        let path2 = entries[1].path.clone();
                        let display1 = path1.display().to_string();
                        let display2 = path2.display().to_string();

                        tracing::info!("파일 비교 시작: {} vs {}", display1, display2);
                        self.ai_state = Some(crate::ai::AiState::loading("파일 비교 중...".to_string()));
                        self.mode = AppMode::AiChat;

                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            let client = crate::ai::AiClient::new(
                                "http://localhost:8080/v1".to_string(),
                                "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                            );

                            match (std::fs::read_to_string(&path1), std::fs::read_to_string(&path2)) {
                                (Ok(content1), Ok(_content2)) => {
                                    let truncated = if content1.len() > 50000 {
                                        format!("{}... (처음 50KB만 비교)", &content1[..50000])
                                    } else {
                                        content1
                                    };
                                    match client.compare_files(truncated, display1, display2).await {
                                        Ok(ai_response) => {
                                            let _ = tx.send(Command::AiResponse(ai_response));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Command::AiError(e.to_string()));
                                        }
                                    }
                                }
                                _ => {
                                    let _ = tx.send(Command::AiError("파일을 읽을 수 없습니다".to_string()));
                                }
                            }
                        });
                    } else {
                        self.ai_state = Some(crate::ai::AiState::error("2개의 파일을 선택해주세요".to_string()));
                        self.mode = AppMode::AiChat;
                    }
                } else {
                    self.ai_state = Some(crate::ai::AiState::error("비교할 파일 2개를 선택하세요 (Insert)".to_string()));
                    self.mode = AppMode::AiChat;
                }
            }

            Command::AiFolderAnalysis => {
                if let Some(entry) = self.active_panel().get_current_entry() {
                    if entry.is_dir {
                        let path = entry.path.clone();
                        let file_path = path.display().to_string();
                        tracing::info!("폴더 분석 시작: {}", file_path);
                        self.ai_state = Some(crate::ai::AiState::loading(format!("폴더 분석: {}", file_path)));
                        self.mode = AppMode::AiChat;

                        let tx = self.tx.clone();
                        tokio::spawn(async move {
                            let client = crate::ai::AiClient::new(
                                "http://localhost:8080/v1".to_string(),
                                "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                            );

                            // 폴더 구조 정보 수집
                            let mut folder_info = format!("폴더: {}\n", file_path);
                            if let Ok(entries) = std::fs::read_dir(&path) {
                                folder_info.push_str("주요 파일/폴더:\n");
                                let mut count = 0;
                                for entry in entries.flatten() {
                                    if count >= 20 {
                                        folder_info.push_str("... (더 많은 항목)\n");
                                        break;
                                    }
                                    if let Ok(name) = entry.file_name().into_string() {
                                        folder_info.push_str(&format!("  - {}\n", name));
                                        count += 1;
                                    }
                                }
                            }

                            match client.analyze_folder(folder_info, file_path).await {
                                Ok(ai_response) => {
                                    let _ = tx.send(Command::AiResponse(ai_response));
                                }
                                Err(e) => {
                                    let _ = tx.send(Command::AiError(e.to_string()));
                                }
                            }
                        });
                    } else {
                        self.ai_state = Some(crate::ai::AiState::error("폴더를 선택해주세요".to_string()));
                        self.mode = AppMode::AiChat;
                    }
                }
            }
            Command::AiResponse(ai_response) => {
                tracing::info!("AI 응답 수신 - thinking: {:?}, result: {} 글자",
                    ai_response.thinking.as_ref().map(|t| t.len()),
                    ai_response.result.len());
                self.ai_state = Some(crate::ai::AiState::new(ai_response));
            }
            Command::AiError(err) => {
                tracing::error!("AI 오류 수신: {}", err);
                self.ai_state = Some(crate::ai::AiState::error(err));
            }
            Command::AiCancel => {
                self.ai_state = None;
                self.mode = AppMode::Normal;
            }
            Command::AiScrollUp => {
                if let Some(ai_state) = &mut self.ai_state {
                    ai_state.scroll_up();
                }
            }
            Command::AiScrollDown => {
                if let Some(ai_state) = &mut self.ai_state {
                    ai_state.scroll_down(20);
                }
            }
            Command::AiPageUp => {
                if let Some(ai_state) = &mut self.ai_state {
                    ai_state.page_up();
                }
            }
            Command::AiPageDown => {
                if let Some(ai_state) = &mut self.ai_state {
                    ai_state.page_down(20);
                }
            }
            Command::AiToggleThinking => {
                if let Some(ai_state) = &mut self.ai_state {
                    ai_state.toggle_thinking();
                }
            }
            Command::AiNaturalCommand => {
                self.dialog = Some(crate::ui::dialog::DialogState::new_ai_command());
                self.mode = AppMode::Dialog;
            }
            Command::AiCommandParsed(ops) => {
                if ops.is_empty() {
                    self.ai_state = Some(crate::ai::AiState::error("파싱된 작업이 없습니다".to_string()));
                    self.mode = AppMode::AiChat;
                } else {
                    self.ai_command_state = Some(crate::ai::AiCommandState::new(ops));
                    self.mode = AppMode::AiCommandConfirm;
                }
            }
            Command::AiCommandConfirm => {
                if let Some(command_state) = &self.ai_command_state {
                    let ops = command_state.ops.clone();
                    let tx = self.tx.clone();
                    let panel_fs = self.active_panel().fs.clone();

                    tokio::spawn(async move {
                        for op in ops {
                            match op {
                                crate::commands::PlannedOp::Delete { path } => {
                                    let is_dir = std::fs::metadata(&path)
                                        .map(|m| m.is_dir())
                                        .unwrap_or(false);
                                    let _ = panel_fs.delete(&path, is_dir);
                                }
                                crate::commands::PlannedOp::Move { from, to } => {
                                    let _ = panel_fs.move_entry(&from, &to);
                                }
                                crate::commands::PlannedOp::Copy { from, to } => {
                                    let is_dir = std::fs::metadata(&from)
                                        .map(|m| m.is_dir())
                                        .unwrap_or(false);
                                    let _ = panel_fs.copy(&from, &to, is_dir);
                                }
                                crate::commands::PlannedOp::Mkdir { path } => {
                                    let _ = panel_fs.mkdir(&path);
                                }
                                crate::commands::PlannedOp::Rename { from, to } => {
                                    let _ = panel_fs.rename(&from, &to);
                                }
                            }
                        }
                        let _ = tx.send(Command::Refresh);
                    });
                }
                self.ai_command_state = None;
                self.mode = AppMode::Normal;
            }
            Command::AiCommandCancel => {
                self.ai_command_state = None;
                self.mode = AppMode::Normal;
            }
            Command::AiCommandScrollUp => {
                if let Some(cmd_state) = &mut self.ai_command_state {
                    cmd_state.scroll_up();
                }
            }
            Command::AiCommandScrollDown => {
                if let Some(cmd_state) = &mut self.ai_command_state {
                    cmd_state.scroll_down();
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
            DialogKind::AiCommand => {
                let nl_command = dialog.input.clone();
                let current_dir = self.active_panel().path.display().to_string();
                let panel = self.active_panel();

                // 현재 폴더의 파일 목록 생성
                let file_listing = panel.entries.iter()
                    .map(|e| format!("{} ({})", e.name, if e.is_dir { "DIR" } else { "FILE" }))
                    .collect::<Vec<_>>()
                    .join("\n");

                // AI 로딩 상태 표시
                self.ai_state = Some(crate::ai::AiState::loading(format!("🔄 명령 해석 중: \"{}\"", nl_command)));
                self.mode = AppMode::AiChat;

                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let client = crate::ai::AiClient::new(
                        "http://localhost:8080/v1".to_string(),
                        "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                    );

                    match client.interpret_command(&nl_command, &current_dir, &file_listing).await {
                        Ok(ai_response) => {
                            // JSON 파싱 시도
                            match parse_planned_ops(&ai_response.result) {
                                Ok(ops) => {
                                    if ops.is_empty() {
                                        let _ = tx.send(Command::AiError(
                                            "❌ 명령 해석 실패: AI가 실행할 작업을 찾을 수 없습니다.\n\n현재 폴더의 파일을 확인하고 다시 시도해주세요.".to_string()
                                        ));
                                    } else {
                                        let _ = tx.send(Command::AiCommandParsed(ops));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Command::AiError(format!(
                                        "❌ 명령 해석 실패\n\nAI가 올바른 작업 목록을 생성하지 못했습니다.\n오류: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Command::AiError(format!(
                                "❌ AI 요청 실패\n\n{}",
                                e
                            )));
                        }
                    }
                });
                return Ok(());
            }
            DialogKind::BatchRename => {
                let pattern = dialog.input.clone();
                let current_dir = self.active_panel().path.display().to_string();
                let panel = self.active_panel();

                // 선택된 파일 목록 생성
                let mut files: Vec<_> = panel.get_selected_entries()
                    .iter()
                    .map(|e| e.path.clone())
                    .collect();
                if files.is_empty() {
                    if let Some(entry) = panel.get_current_entry() {
                        files.push(entry.path.clone());
                    }
                }
                if files.is_empty() {
                    return Ok(());
                }

                let file_listing = files.iter()
                    .map(|p| format!("{}", p.display()))
                    .collect::<Vec<_>>()
                    .join("\n");
                tracing::debug!("배치 리네이밍 파일 목록:\n{}", file_listing);
                tracing::debug!("배치 리네이밍 패턴: {}", pattern);

                // AI 로딩 상태 표시
                self.ai_state = Some(crate::ai::AiState::loading(format!("🔄 이름 변경 패턴 분석 중: \"{}\"", pattern)));
                self.mode = AppMode::AiChat;

                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let client = crate::ai::AiClient::new(
                        "http://localhost:8080/v1".to_string(),
                        "Qwen_Qwen3.6-35B-A3B-Q4_0.gguf".to_string(),
                    );

                    match client.batch_rename(&pattern, &current_dir, &file_listing).await {
                        Ok(ai_response) => {
                            match parse_planned_ops(&ai_response.result) {
                                Ok(ops) => {
                                    if ops.is_empty() {
                                        let _ = tx.send(Command::AiError(
                                            "❌ 이름 변경 실패: 변경할 파일을 찾지 못했습니다.\n\n패턴을 다시 입력하고 시도해주세요.".to_string()
                                        ));
                                    } else {
                                        let _ = tx.send(Command::AiCommandParsed(ops));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(Command::AiError(format!(
                                        "❌ 이름 변경 실패\n\nAI가 올바른 파일명 목록을 생성하지 못했습니다.\n오류: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Command::AiError(format!(
                                "❌ AI 요청 실패\n\n{}",
                                e
                            )));
                        }
                    }
                });
                return Ok(());
            }
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

fn parse_planned_ops(json_str: &str) -> Result<Vec<crate::commands::PlannedOp>> {
    use std::path::PathBuf;

    let trimmed = json_str.trim();
    tracing::debug!("JSON 파싱 입력 ({}글자): {}", trimmed.len(), trimmed);

    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("AI 응답이 비어있습니다. 서버가 응답을 반환하지 않았을 수 있습니다."));
    }

    // JSON이 마크다운 코드블록에 감싸져 있을 수 있으므로 추출
    let json_text = if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        let start = lines.iter().position(|l| l.starts_with("```") && l.contains("json"))
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = lines.iter().rposition(|l| l.starts_with("```"))
            .unwrap_or(lines.len());
        lines[start..end].join("\n")
    } else {
        // JSON 배열을 찾기 (앞에 설명이 있을 수 있음)
        if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed.rfind(']') {
                trimmed[start..=end].to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        }
    };

    let final_json = json_text.trim();
    tracing::debug!("정제된 JSON: {}", final_json);

    let operations: Vec<serde_json::Value> = serde_json::from_str(final_json)
        .map_err(|e| anyhow::anyhow!("JSON 파싱 실패: {}", e))?;

    let mut ops = Vec::new();
    for op_value in operations {
        let op_type = op_value.get("op")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("'op' 필드가 없습니다"))?;

        match op_type {
            "delete" => {
                let path = op_value.get("path")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("delete: 'path' 필드 필요"))?;
                ops.push(crate::commands::PlannedOp::Delete { path });
            }
            "move" => {
                let from = op_value.get("from")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("move: 'from' 필드 필요"))?;
                let to = op_value.get("to")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("move: 'to' 필드 필요"))?;
                ops.push(crate::commands::PlannedOp::Move { from, to });
            }
            "copy" => {
                let from = op_value.get("from")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("copy: 'from' 필드 필요"))?;
                let to = op_value.get("to")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("copy: 'to' 필드 필요"))?;
                ops.push(crate::commands::PlannedOp::Copy { from, to });
            }
            "mkdir" => {
                let path = op_value.get("path")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("mkdir: 'path' 필드 필요"))?;
                ops.push(crate::commands::PlannedOp::Mkdir { path });
            }
            "rename" => {
                let from = op_value.get("from")
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("rename: 'from' 필드 필요"))?;
                let to = op_value.get("to")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("rename: 'to' 필드 필요"))?;
                ops.push(crate::commands::PlannedOp::Rename { from, to });
            }
            _ => {
                tracing::warn!("미지의 작업 타입: {}", op_type);
            }
        }
    }

    Ok(ops)
}
