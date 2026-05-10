use crate::fs::{FileEntry, FileSystem};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortBy {
    Name,
    Size,
    Modified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    None,
    Quick,
    Wildcard,
    Find,
}

pub struct PanelState {
    pub path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub cursor: usize,
    pub selected: HashSet<usize>,
    pub sort_by: SortBy,
    pub reverse: bool,
    pub fs: std::sync::Arc<Box<dyn FileSystem>>,
    pub filter_query: Option<String>,
    pub filtered_entries: Vec<FileEntry>,
    pub search_mode: SearchMode,
    pub archive_base: Option<PathBuf>,
    pub show_hidden: bool,
    pub list_total_size: u64,
    pub recursive_total_size: Option<u64>,
    pub is_calculating: bool,
}

impl PanelState {
    pub fn new(fs: std::sync::Arc<Box<dyn FileSystem>>) -> Result<Self> {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));

        let mut state = PanelState {
            path: home,
            entries: Vec::new(),
            cursor: 0,
            selected: HashSet::new(),
            sort_by: SortBy::Name,
            reverse: false,
            fs,
            filter_query: None,
            filtered_entries: Vec::new(),
            search_mode: SearchMode::None,
            archive_base: None,
            show_hidden: false,
            list_total_size: 0,
            recursive_total_size: None,
            is_calculating: false,
        };
        state.refresh()?;
        Ok(state)
    }

    pub fn refresh(&mut self) -> Result<()> {
        let mut entries = self.fs.list(&self.path)?;
        
        if !self.show_hidden {
            entries.retain(|e| !e.name.starts_with('.'));
        }

        self.entries = entries;
        self.apply_sort();
        self.cursor = self.cursor.min(self.entries.len().saturating_sub(1));
        
        // 1차: 현재 목록 합계 즉시 계산
        self.list_total_size = self.entries.iter()
            .filter(|e| !e.is_dir)
            .map(|e| e.size)
            .sum();
        
        // 새로운 경로로 오면 재귀 합계 초기화
        self.recursive_total_size = None;
        
        Ok(())
    }

    pub fn set_show_hidden(&mut self, show: bool) -> Result<()> {
        self.show_hidden = show;
        self.refresh()
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> Result<()> {
        if self.fs.is_dir(&path) {
            self.path = path;
            self.cursor = 0;
            self.selected.clear();
            self.clear_filter(); // 내비게이션 시 필터 해제
            self.refresh()?;
        }
        Ok(())
    }

    pub fn go_parent(&mut self) -> Result<()> {
        if let Some(archive_base) = &self.archive_base {
            if self.path == PathBuf::from("") || self.path == PathBuf::from("/") {
                // We are at archive root, switch back to local FS
                let parent = archive_base.parent().unwrap_or_else(|| Path::new("/")).to_path_buf();
                self.archive_base = None;
                self.fs = std::sync::Arc::new(crate::fs::create_local_fs());
                self.path = parent;
                self.cursor = 0;
                self.selected.clear();
                self.clear_filter(); // 아카이브 탈출 시 필터 해제
                self.refresh()?;
                return Ok(());
            }
        }

        if let Some(parent) = self.path.parent() {
            self.navigate_to(parent.to_path_buf())?;
        }
        Ok(())
    }

    pub fn set_fs(&mut self, fs: std::sync::Arc<Box<dyn FileSystem>>, path: PathBuf, is_archive: bool) -> Result<()> {
        self.fs = fs;
        self.path = path;
        if is_archive {
            self.archive_base = Some(self.path.clone());
            self.path = PathBuf::from(""); // Root inside archive
        } else {
            self.archive_base = None;
        }
        self.cursor = 0;
        self.selected.clear();
        self.clear_filter(); // FS 전환 시 필터 해제
        self.refresh()?;
        Ok(())
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        let max = self.visible_entries().len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.cursor = self.cursor.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        let max = self.visible_entries().len().saturating_sub(1);
        self.cursor = (self.cursor + page_size).min(max);
    }

    pub fn toggle_select(&mut self) {
        if self.visible_entries().get(self.cursor).is_some() {
            if self.selected.contains(&self.cursor) {
                self.selected.remove(&self.cursor);
            } else {
                self.selected.insert(self.cursor);
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selected = (0..self.visible_entries().len()).collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn get_selected_entries(&self) -> Vec<&FileEntry> {
        self.selected
            .iter()
            .filter_map(|&idx| self.entries.get(idx))
            .collect()
    }

    pub fn get_current_entry(&self) -> Option<&FileEntry> {
        self.visible_entries().get(self.cursor)
    }

    pub fn apply_sort(&mut self) {
        self.entries.sort_by(|a, b| {
            let cmp = match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Modified => a.modified.cmp(&b.modified),
            };

            if self.reverse {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    pub fn visible_entries(&self) -> &[FileEntry] {
        if self.filter_query.is_some() {
            &self.filtered_entries
        } else {
            &self.entries
        }
    }

    pub fn apply_quick_filter(&mut self, query: &str) {
        self.filter_query = Some(query.to_string());
        self.search_mode = SearchMode::Quick;
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect();
        self.cursor = 0;
    }

    pub fn apply_wildcard_filter(&mut self, pattern: &str) {
        use glob::Pattern as GlobPattern;

        self.filter_query = Some(pattern.to_string());
        self.search_mode = SearchMode::Wildcard;

        if let Ok(glob_pattern) = GlobPattern::new(pattern) {
            self.filtered_entries = self
                .entries
                .iter()
                .filter(|e| glob_pattern.matches(&e.name))
                .cloned()
                .collect();
        }
        self.cursor = 0;
    }

    pub fn set_find_results(&mut self, results: Vec<FileEntry>) {
        self.filter_query = Some("[Find Results]".to_string());
        self.search_mode = SearchMode::Find;
        self.filtered_entries = results;
        self.cursor = 0;
    }

    pub fn clear_filter(&mut self) {
        self.filter_query = None;
        self.filtered_entries.clear();
        self.search_mode = SearchMode::None;
        self.cursor = 0;
    }
}
