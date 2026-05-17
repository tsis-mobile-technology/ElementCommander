use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use crate::fs::FileEntry;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DiffEntry {
    pub relative_path: String,
    pub left_size: Option<u64>,
    pub right_size: Option<u64>,
    pub left_modified: Option<u64>,
    pub right_modified: Option<u64>,
}

pub struct SyncReport {
    pub left_root: String,
    pub right_root: String,
    pub only_left: Vec<DiffEntry>,
    pub only_right: Vec<DiffEntry>,
    pub different: Vec<DiffEntry>,
    pub same_count: usize,
}

pub fn analyze_sync(left: &Path, right: &Path) -> Result<SyncReport> {
    // Collect all files from left path with relative paths
    let mut left_map: HashMap<String, (u64, u64)> = HashMap::new();
    for entry in WalkDir::new(left)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Ok(metadata) = entry.metadata() {
            if !metadata.is_dir() {
                if let Ok(rel_path) = entry.path().strip_prefix(left) {
                    let rel_str = rel_path.to_string_lossy().to_string();
                    let size = metadata.len();
                    let mtime = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    left_map.insert(rel_str, (size, mtime));
                }
            }
        }
    }

    // Collect all files from right path with relative paths
    let mut right_map: HashMap<String, (u64, u64)> = HashMap::new();
    for entry in WalkDir::new(right)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Ok(metadata) = entry.metadata() {
            if !metadata.is_dir() {
                if let Ok(rel_path) = entry.path().strip_prefix(right) {
                    let rel_str = rel_path.to_string_lossy().to_string();
                    let size = metadata.len();
                    let mtime = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    right_map.insert(rel_str, (size, mtime));
                }
            }
        }
    }

    let mut only_left = Vec::new();
    let mut only_right = Vec::new();
    let mut different = Vec::new();
    let mut same_count = 0;

    // Find files only in left or different
    for (rel_path, (left_size, left_mtime)) in &left_map {
        if let Some((right_size, right_mtime)) = right_map.get(rel_path) {
            if left_size == right_size && left_mtime == right_mtime {
                same_count += 1;
            } else {
                different.push(DiffEntry {
                    relative_path: rel_path.clone(),
                    left_size: Some(*left_size),
                    right_size: Some(*right_size),
                    left_modified: Some(*left_mtime),
                    right_modified: Some(*right_mtime),
                });
            }
        } else {
            only_left.push(DiffEntry {
                relative_path: rel_path.clone(),
                left_size: Some(*left_size),
                right_size: None,
                left_modified: Some(*left_mtime),
                right_modified: None,
            });
        }
    }

    // Find files only in right
    for (rel_path, (right_size, right_mtime)) in &right_map {
        if !left_map.contains_key(rel_path) {
            only_right.push(DiffEntry {
                relative_path: rel_path.clone(),
                left_size: None,
                right_size: Some(*right_size),
                left_modified: None,
                right_modified: Some(*right_mtime),
            });
        }
    }

    // Sort by relative path for consistent output
    only_left.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    only_right.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    different.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(SyncReport {
        left_root: left.display().to_string(),
        right_root: right.display().to_string(),
        only_left,
        only_right,
        different,
        same_count,
    })
}

pub fn format_report(report: &SyncReport) -> String {
    let mut result = format!(
        "폴더 동기화 분석\n\n왼쪽: {}\n오른쪽: {}\n\n",
        report.left_root, report.right_root
    );

    result.push_str(&format!(
        "일치하는 파일: {} 개\n\n",
        report.same_count
    ));

    if !report.only_left.is_empty() {
        result.push_str(&format!("■ 왼쪽에만 있는 파일 ({} 개)\n", report.only_left.len()));
        for entry in report.only_left.iter().take(20) {
            let size_str = entry
                .left_size
                .map(|s| FileEntry::format_size_static(s))
                .unwrap_or_default();
            result.push_str(&format!("  {} ({})\n", entry.relative_path, size_str));
        }
        if report.only_left.len() > 20 {
            result.push_str(&format!("  ... 외 {} 개\n", report.only_left.len() - 20));
        }
        result.push('\n');
    }

    if !report.only_right.is_empty() {
        result.push_str(&format!("■ 오른쪽에만 있는 파일 ({} 개)\n", report.only_right.len()));
        for entry in report.only_right.iter().take(20) {
            let size_str = entry
                .right_size
                .map(|s| FileEntry::format_size_static(s))
                .unwrap_or_default();
            result.push_str(&format!("  {} ({})\n", entry.relative_path, size_str));
        }
        if report.only_right.len() > 20 {
            result.push_str(&format!("  ... 외 {} 개\n", report.only_right.len() - 20));
        }
        result.push('\n');
    }

    if !report.different.is_empty() {
        result.push_str(&format!(
            "■ 양쪽에 있지만 다른 파일 ({} 개)\n",
            report.different.len()
        ));
        for entry in report.different.iter().take(20) {
            let left_size = entry
                .left_size
                .map(|s| FileEntry::format_size_static(s))
                .unwrap_or_default();
            let right_size = entry
                .right_size
                .map(|s| FileEntry::format_size_static(s))
                .unwrap_or_default();
            result.push_str(&format!(
                "  {} (왼쪽: {}, 오른쪽: {})\n",
                entry.relative_path, left_size, right_size
            ));
        }
        if report.different.len() > 20 {
            result.push_str(&format!("  ... 외 {} 개\n", report.different.len() - 20));
        }
    }

    result
}
