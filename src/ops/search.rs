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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use chrono::{Duration, TimeZone};

    #[test]
    fn test_search_criteria_deserialization() {
        let json = r#"{
            "pattern": "test",
            "extension": "rs",
            "min_size": 100,
            "max_size": 1000,
            "modified_after": "2026-05-10T12:00:00Z",
            "modified_before": "2026-05-20T12:00:00Z"
        }"#;

        let criteria: SearchCriteria = serde_json::from_str(json).unwrap();
        assert_eq!(criteria.pattern, Some("test".to_string()));
        assert_eq!(criteria.extension, Some("rs".to_string()));
        assert_eq!(criteria.min_size, Some(100));
        assert_eq!(criteria.max_size, Some(1000));
        assert!(criteria.modified_after.is_some());
    }

    #[test]
    fn test_find_files_with_criteria() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let criteria = SearchCriteria {
            pattern: Some("test".to_string()),
            extension: Some("rs".to_string()),
            ..Default::default()
        };

        let results = find_files_with_criteria(dir.path(), &criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_file.rs");
    }

    #[test]
    fn test_find_files_size_filter() {
        let dir = tempdir().unwrap();
        
        let small_file = dir.path().join("small.txt");
        let mut f1 = File::create(&small_file).unwrap();
        f1.write_all(&[0; 10]).unwrap(); // 10 bytes

        let large_file = dir.path().join("large.txt");
        let mut f2 = File::create(&large_file).unwrap();
        f2.write_all(&[0; 1000]).unwrap(); // 1000 bytes

        let criteria = SearchCriteria {
            min_size: Some(500),
            ..Default::default()
        };

        let results = find_files_with_criteria(dir.path(), &criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "large.txt");
    }
}
