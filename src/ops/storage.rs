use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct DirSizeEntry {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}

pub struct StorageReport {
    pub root: String,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
    pub top_dirs: Vec<DirSizeEntry>,
    pub large_files: Vec<FileEntry>,
}

pub fn analyze_storage(root: &Path) -> Result<StorageReport> {
    if !root.exists() {
        return Ok(StorageReport {
            root: root.display().to_string(),
            total_size: 0,
            total_files: 0,
            total_dirs: 0,
            top_dirs: vec![],
            large_files: vec![],
        });
    }

    let mut total_size = 0u64;
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut all_files = Vec::new();

    // 모든 파일 수집 및 통계
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                total_size = total_size.saturating_add(size);
                total_files += 1;
                all_files.push(FileEntry {
                    path: path.to_path_buf(),
                    size,
                });
            }
        } else if path.is_dir() && path != root {
            total_dirs += 1;
        }
    }

    // 큰 파일 Top 20
    all_files.sort_by(|a, b| b.size.cmp(&a.size));
    let large_files: Vec<FileEntry> = all_files.into_iter().take(20).collect();

    // 직속 하위 폴더별 크기 Top 10
    let mut dir_sizes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let size = calculate_dir_size(&path);
                dir_sizes.push(DirSizeEntry { path, size });
            }
        }
    }

    dir_sizes.sort_by(|a, b| b.size.cmp(&a.size));
    let top_dirs: Vec<DirSizeEntry> = dir_sizes.into_iter().take(10).collect();

    Ok(StorageReport {
        root: root.display().to_string(),
        total_size,
        total_files,
        total_dirs,
        top_dirs,
        large_files,
    })
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&entry_path) {
                    size = size.saturating_add(metadata.len());
                }
            } else if entry_path.is_dir() {
                size = size.saturating_add(calculate_dir_size(&entry_path));
            }
        }
    }
    size
}

pub fn format_report(report: &StorageReport) -> String {
    let mut result = format!(
        "경로: {}\n총 크기: {}\n파일: {}\n폴더: {}\n\n",
        report.root,
        crate::fs::FileEntry::format_size_static(report.total_size),
        report.total_files,
        report.total_dirs
    );

    if !report.top_dirs.is_empty() {
        result.push_str("■ 큰 폴더 Top 10 (하위 포함)\n");
        for entry in &report.top_dirs {
            let name = entry.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            result.push_str(&format!(
                "  {} ({})\n",
                name,
                crate::fs::FileEntry::format_size_static(entry.size)
            ));
        }
        result.push('\n');
    }

    if !report.large_files.is_empty() {
        result.push_str("■ 가장 큰 파일 Top 20\n");
        for entry in &report.large_files {
            let name = entry.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            result.push_str(&format!(
                "  {} ({})\n",
                name,
                crate::fs::FileEntry::format_size_static(entry.size)
            ));
        }
    }

    result
}
