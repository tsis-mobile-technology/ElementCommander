pub mod local;
pub mod archive;

use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: DateTime<Local>,
    #[allow(dead_code)]
    pub permissions: u32,
}

impl FileEntry {
    pub fn display_size(&self) -> String {
        if self.is_dir {
            return String::from("<DIR>");
        }
        Self::format_size(self.size)
    }

    pub fn format_size(size: u64) -> String {
        Self::format_size_static(size)
    }

    pub fn format_size_static(size: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        let mut size = size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        if unit_idx == 0 {
            format!("{} B", size as u64)
        } else {
            format!("{:.1} {}", size, UNITS[unit_idx])
        }
    }

    pub fn display_modified(&self) -> String {
        self.modified.format("%Y-%m-%d %H:%M").to_string()
    }
}

pub trait FileSystem: Send + Sync {
    fn list(&self, path: &Path) -> Result<Vec<FileEntry>>;
    fn copy(&self, src: &Path, dst: &Path, recursive: bool) -> Result<()>;
    fn move_entry(&self, src: &Path, dst: &Path) -> Result<()>;
    fn delete(&self, path: &Path, recursive: bool) -> Result<()>;
    fn mkdir(&self, path: &Path) -> Result<()>;
    fn rename(&self, path: &Path, new_name: &str) -> Result<()>;
    #[allow(dead_code)]
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
}

pub fn create_local_fs() -> Box<dyn FileSystem> {
    Box::new(local::LocalFs)
}

pub fn create_archive_fs(path: PathBuf) -> Result<Box<dyn FileSystem>> {
    Ok(Box::new(archive::ArchiveFs::new(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(FileEntry::format_size_static(0), "0 B");
        assert_eq!(FileEntry::format_size_static(512), "512 B");
        assert_eq!(FileEntry::format_size_static(1024), "1.0 K");
        assert_eq!(FileEntry::format_size_static(1024 * 1024), "1.0 M");
        assert_eq!(FileEntry::format_size_static(1024 * 1024 * 1024), "1.0 G");
        assert_eq!(FileEntry::format_size_static(1024 * 1024 * 1024 * 1024), "1.0 T");
        assert_eq!(FileEntry::format_size_static(1500), "1.5 K");
    }
}
