use crate::fs::FileEntry;
use std::path::Path;
use walkdir::WalkDir;
use chrono::{Local, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchCriteria {
    pub pattern: Option<String>,
    pub extension: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub modified_after: Option<DateTime<Local>>,
    pub modified_before: Option<DateTime<Local>>,
}

pub fn find_files(root: &Path, query: &str) -> Vec<FileEntry> {
    let criteria = SearchCriteria {
        pattern: Some(query.to_string()),
        ..Default::default()
    };
    find_files_with_criteria(root, &criteria)
}

pub fn find_files_with_criteria(root: &Path, criteria: &SearchCriteria) -> Vec<FileEntry> {
    let mut results = Vec::new();
    let pattern_lower = criteria.pattern.as_ref().map(|p| p.to_lowercase());
    let ext_lower = criteria.extension.as_ref().map(|e| e.to_lowercase().trim_start_matches('.').to_string());

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .skip(1)
    {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let file_name_lower = file_name.to_lowercase();

        // 1. 패턴 매칭 (단순 포함 검색)
        if let Some(ref p) = pattern_lower {
            if !file_name_lower.contains(p) {
                continue;
            }
        }

        // 2. 확장자 매칭
        if let Some(ref target_ext) = ext_lower {
            let actual_ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            if actual_ext != *target_ext {
                continue;
            }
        }

        if let Ok(metadata) = path.metadata() {
            let is_dir = metadata.is_dir();
            let size = metadata.len();
            let modified: DateTime<Local> = metadata.modified()
                .ok()
                .and_then(|t| t.elapsed().ok())
                .map(|elapsed| Local::now() - chrono::Duration::from_std(elapsed).unwrap_or_default())
                .unwrap_or_else(Local::now);

            // 3. 크기 필터
            if let Some(min) = criteria.min_size {
                if size < min { continue; }
            }
            if let Some(max) = criteria.max_size {
                if size > max { continue; }
            }

            // 4. 수정일 필터
            if let Some(after) = criteria.modified_after {
                if modified < after { continue; }
            }
            if let Some(before) = criteria.modified_before {
                if modified > before { continue; }
            }

            results.push(FileEntry {
                name: file_name.to_string(),
                path: path.to_path_buf(),
                is_dir,
                size: if is_dir { 0 } else { size },
                modified,
                permissions: 0o644,
            });
        }
    }

    results
}
