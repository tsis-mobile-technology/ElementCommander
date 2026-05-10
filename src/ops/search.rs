use crate::fs::FileEntry;
use std::path::Path;
use walkdir::WalkDir;
use chrono::Local;

pub fn find_files(root: &Path, query: &str) -> Vec<FileEntry> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .skip(1) // skip the root directory itself
    {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Simple substring match (case-insensitive)
        if file_name.to_lowercase().contains(&query_lower) {
            if let Ok(metadata) = path.metadata() {
                let is_dir = metadata.is_dir();
                results.push(FileEntry {
                    name: file_name.to_string(),
                    path: path.to_path_buf(),
                    is_dir,
                    size: if is_dir { 0 } else { metadata.len() },
                    modified: metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|elapsed| Local::now() - chrono::Duration::from_std(elapsed).unwrap_or_default())
                        .unwrap_or_else(Local::now),
                    permissions: 0o644, // simplified permissions
                });
            }
        }
    }

    results
}
