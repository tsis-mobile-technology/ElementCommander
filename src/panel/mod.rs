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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FileSystem;
    use chrono::{Duration, TimeZone, Local};
    use std::sync::Arc;

    struct MockFs {
        entries: Vec<FileEntry>,
    }

    impl MockFs {
        fn new() -> Self {
            let now = Local.with_ymd_and_hms(2026, 5, 16, 12, 0, 0).unwrap();
            Self {
                entries: vec![
                    FileEntry {
                        name: "file3.txt".to_string(),
                        path: PathBuf::from("file3.txt"),
                        is_dir: false,
                        size: 300,
                        modified: now - Duration::days(1),
                        permissions: 0,
                    },
                    FileEntry {
                        name: "file1.txt".to_string(),
                        path: PathBuf::from("file1.txt"),
                        is_dir: false,
                        size: 100,
                        modified: now - Duration::days(3),
                        permissions: 0,
                    },
                    FileEntry {
                        name: "file2.txt".to_string(),
                        path: PathBuf::from("file2.txt"),
                        is_dir: false,
                        size: 200,
                        modified: now - Duration::days(2),
                        permissions: 0,
                    },
                    FileEntry {
                        name: "dir1".to_string(),
                        path: PathBuf::from("dir1"),
                        is_dir: true,
                        size: 0,
                        modified: now,
                        permissions: 0,
                    },
                ],
            }
        }
    }

    impl FileSystem for MockFs {
        fn list(&self, _path: &Path) -> Result<Vec<FileEntry>> {
            Ok(self.entries.clone())
        }
        fn copy(&self, _src: &Path, _dst: &Path, _recursive: bool) -> Result<()> { Ok(()) }
        fn move_entry(&self, _src: &Path, _dst: &Path) -> Result<()> { Ok(()) }
        fn delete(&self, _path: &Path, _recursive: bool) -> Result<()> { Ok(()) }
        fn mkdir(&self, _path: &Path) -> Result<()> { Ok(()) }
        fn rename(&self, _path: &Path, _new_name: &str) -> Result<()> { Ok(()) }
        fn exists(&self, _path: &Path) -> bool { true }
        fn is_dir(&self, _path: &Path) -> bool { false }
    }

    fn setup_panel() -> PanelState {
        let fs = Arc::new(Box::new(MockFs::new()) as Box<dyn FileSystem>);
        PanelState {
            path: PathBuf::from("/"),
            entries: MockFs::new().entries,
            cursor: 0,
            selected: HashSet::new(),
            sort_by: SortBy::Name,
            reverse: false,
            fs,
            filter_query: None,
            filtered_entries: Vec::new(),
            search_mode: SearchMode::None,
            archive_base: None,
            show_hidden: true,
            list_total_size: 0,
            recursive_total_size: None,
            is_calculating: false,
        }
    }

    #[test]
    fn test_sorting() {
        let mut panel = setup_panel();
        
        // Sort by Name
        panel.sort_by = SortBy::Name;
        panel.apply_sort();
        assert_eq!(panel.entries[0].name, "dir1");
        assert_eq!(panel.entries[1].name, "file1.txt");
        assert_eq!(panel.entries[2].name, "file2.txt");
        assert_eq!(panel.entries[3].name, "file3.txt");

        // Sort by Size
        panel.sort_by = SortBy::Size;
        panel.apply_sort();
        assert_eq!(panel.entries[0].name, "dir1"); // size 0
        assert_eq!(panel.entries[1].name, "file1.txt"); // size 100
        assert_eq!(panel.entries[2].name, "file2.txt"); // size 200
        assert_eq!(panel.entries[3].name, "file3.txt"); // size 300

        // Reverse Sort
        panel.reverse = true;
        panel.apply_sort();
        assert_eq!(panel.entries[0].name, "file3.txt");
    }

    #[test]
    fn test_quick_filter() {
        let mut panel = setup_panel();
        panel.apply_quick_filter("file");
        assert_eq!(panel.visible_entries().len(), 3);
        assert!(panel.visible_entries().iter().all(|e| e.name.contains("file")));

        panel.apply_quick_filter("file1");
        assert_eq!(panel.visible_entries().len(), 1);
        assert_eq!(panel.visible_entries()[0].name, "file1.txt");

        panel.clear_filter();
        assert_eq!(panel.visible_entries().len(), 4);
    }

    #[test]
    fn test_wildcard_filter() {
        let mut panel = setup_panel();
        panel.apply_wildcard_filter("*.txt");
        assert_eq!(panel.visible_entries().len(), 3);

        panel.apply_wildcard_filter("dir*");
        assert_eq!(panel.visible_entries().len(), 1);
        assert_eq!(panel.visible_entries()[0].name, "dir1");
    }

    #[test]
    fn test_selection() {
        let mut panel = setup_panel();
        panel.cursor = 1;
        panel.toggle_select();
        assert!(panel.selected.contains(&1));
        assert_eq!(panel.get_selected_entries().len(), 1);

        panel.select_all();
        assert_eq!(panel.selected.len(), 4);

        panel.clear_selection();
        assert_eq!(panel.selected.len(), 0);
    }
}
