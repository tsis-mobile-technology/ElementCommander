use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

pub struct AgeGroup {
    pub label: &'static str,
    pub files: Vec<(PathBuf, u64)>,
    pub total_size: u64,
    pub file_count: usize,
}

pub struct CleanupReport {
    pub root: String,
    pub groups: Vec<AgeGroup>,
    pub total_files: usize,
    pub total_size: u64,
}

pub fn analyze_old_files(root: &Path) -> Result<CleanupReport> {
    if !root.exists() {
        return Ok(CleanupReport {
            root: root.display().to_string(),
            groups: vec![],
            total_files: 0,
            total_size: 0,
        });
    }

    let now = SystemTime::now();
    let mut group_recent = AgeGroup { label: "30일 이내", files: Vec::new(), total_size: 0, file_count: 0 };
    let mut group_recent_90 = AgeGroup { label: "30~90일", files: Vec::new(), total_size: 0, file_count: 0 };
    let mut group_old_365 = AgeGroup { label: "90~365일", files: Vec::new(), total_size: 0, file_count: 0 };
    let mut group_very_old = AgeGroup { label: "365일 이상 (정리 대상)", files: Vec::new(), total_size: 0, file_count: 0 };

    let mut total_files = 0;
    let mut total_size = 0u64;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                total_files += 1;
                total_size = total_size.saturating_add(size);

                // 파일 수정 시간으로부터 경과 일수 계산
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = now.duration_since(modified) {
                        let days = duration.as_secs() / 86400;

                        let target = if days < 30 {
                            &mut group_recent
                        } else if days < 90 {
                            &mut group_recent_90
                        } else if days < 365 {
                            &mut group_old_365
                        } else {
                            &mut group_very_old
                        };

                        target.files.push((path.to_path_buf(), size));
                        target.total_size = target.total_size.saturating_add(size);
                        target.file_count += 1;
                    }
                }
            }
        }
    }

    let groups = vec![group_recent, group_recent_90, group_old_365, group_very_old];

    Ok(CleanupReport {
        root: root.display().to_string(),
        groups,
        total_files,
        total_size,
    })
}

pub fn format_report(report: &CleanupReport) -> String {
    let mut result = format!("경로: {}\n총 파일: {}\n전체 크기: {}\n\n",
        report.root,
        report.total_files,
        crate::fs::FileEntry::format_size_static(report.total_size));

    for group in &report.groups {
        if group.file_count == 0 {
            continue;
        }

        result.push_str(&format!("■ {} ({} 파일, {})\n",
            group.label,
            group.file_count,
            crate::fs::FileEntry::format_size_static(group.total_size)));

        // 상위 10개 파일만 표시
        for (path, size) in group.files.iter().take(10) {
            result.push_str(&format!("  - {} ({})\n",
                path.display(),
                crate::fs::FileEntry::format_size_static(*size)));
        }

        if group.file_count > 10 {
            result.push_str(&format!("  ... 외 {} 파일\n", group.file_count - 10));
        }
        result.push('\n');
    }

    result
}
