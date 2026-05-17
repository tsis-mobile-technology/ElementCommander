use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Image,
    Video,
    Audio,
    Document,
    Code,
    Archive,
    Data,
    Other,
}

#[allow(dead_code)]
pub struct FileGroup {
    pub type_name: &'static str,
    pub file_type: FileType,
    pub extensions: &'static [&'static str],
    pub files: Vec<(PathBuf, u64)>,
    pub total_size: u64,
    pub count: usize,
}

pub struct ClassifyReport {
    pub root: String,
    pub groups: Vec<FileGroup>,
    pub total_files: usize,
    pub total_size: u64,
}

const IMAGES: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "tiff", "ico", "heic"];
const VIDEOS: &[&str] = &["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", "m4v", "mpg", "mpeg"];
const AUDIOS: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "wma", "m4a", "opus", "aiff"];
const DOCUMENTS: &[&str] = &["pdf", "doc", "docx", "ppt", "pptx", "xls", "xlsx", "txt", "md", "rst", "pages"];
const CODES: &[&str] = &["rs", "py", "js", "ts", "jsx", "tsx", "go", "c", "cpp", "h", "java", "rb", "php", "swift", "kt"];
const ARCHIVES: &[&str] = &["zip", "tar", "gz", "bz2", "rar", "7z", "xz", "zst"];
const DATA: &[&str] = &["json", "yaml", "yml", "toml", "csv", "xml", "sql", "db", "sqlite"];

fn get_file_type(ext: &str) -> FileType {
    let ext_lower = ext.to_lowercase();
    if IMAGES.contains(&ext_lower.as_str()) {
        FileType::Image
    } else if VIDEOS.contains(&ext_lower.as_str()) {
        FileType::Video
    } else if AUDIOS.contains(&ext_lower.as_str()) {
        FileType::Audio
    } else if DOCUMENTS.contains(&ext_lower.as_str()) {
        FileType::Document
    } else if CODES.contains(&ext_lower.as_str()) {
        FileType::Code
    } else if ARCHIVES.contains(&ext_lower.as_str()) {
        FileType::Archive
    } else if DATA.contains(&ext_lower.as_str()) {
        FileType::Data
    } else {
        FileType::Other
    }
}

pub fn classify_files(root: &Path) -> Result<ClassifyReport> {
    if !root.exists() {
        return Ok(ClassifyReport {
            root: root.display().to_string(),
            groups: vec![],
            total_files: 0,
            total_size: 0,
        });
    }

    let mut groups = vec![
        FileGroup { type_name: "이미지", file_type: FileType::Image, extensions: IMAGES, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "동영상", file_type: FileType::Video, extensions: VIDEOS, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "음악", file_type: FileType::Audio, extensions: AUDIOS, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "문서", file_type: FileType::Document, extensions: DOCUMENTS, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "코드", file_type: FileType::Code, extensions: CODES, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "압축", file_type: FileType::Archive, extensions: ARCHIVES, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "데이터", file_type: FileType::Data, extensions: DATA, files: Vec::new(), total_size: 0, count: 0 },
        FileGroup { type_name: "기타", file_type: FileType::Other, extensions: &[], files: Vec::new(), total_size: 0, count: 0 },
    ];

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

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let file_type = get_file_type(&ext);

                let group_idx = match file_type {
                    FileType::Image => 0,
                    FileType::Video => 1,
                    FileType::Audio => 2,
                    FileType::Document => 3,
                    FileType::Code => 4,
                    FileType::Archive => 5,
                    FileType::Data => 6,
                    FileType::Other => 7,
                };

                groups[group_idx].files.push((path.to_path_buf(), size));
                groups[group_idx].total_size = groups[group_idx].total_size.saturating_add(size);
                groups[group_idx].count += 1;
            }
        }
    }

    // count == 0인 그룹 제외
    groups.retain(|g| g.count > 0);

    Ok(ClassifyReport {
        root: root.display().to_string(),
        groups,
        total_files,
        total_size,
    })
}

pub fn format_report(report: &ClassifyReport) -> String {
    let mut result = format!("경로: {}\n총 파일: {}\n전체 크기: {}\n\n",
        report.root,
        report.total_files,
        crate::fs::FileEntry::format_size_static(report.total_size));

    for group in &report.groups {
        result.push_str(&format!("■ {} ({} 파일, {})\n",
            group.type_name,
            group.count,
            crate::fs::FileEntry::format_size_static(group.total_size)));

        // 상위 5개 파일만 표시
        for (path, size) in group.files.iter().take(5) {
            result.push_str(&format!("  - {} ({})\n",
                path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                crate::fs::FileEntry::format_size_static(*size)));
        }

        if group.count > 5 {
            result.push_str(&format!("  ... 외 {} 파일\n", group.count - 5));
        }
        result.push('\n');
    }

    result
}
