use super::{FileEntry, FileSystem};
use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::Path;

pub struct LocalFs;

impl FileSystem for LocalFs {
    fn list(&self, path: &Path) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        if !path.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();

            let modified = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let modified = DateTime::<Local>::from(std::time::UNIX_EPOCH + std::time::Duration::from_secs(modified));

            entries.push(FileEntry {
                name,
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified,
                permissions: 0, // TODO: Extract actual permissions
            });
        }

        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });

        Ok(entries)
    }

    fn copy(&self, src: &Path, dst: &Path, recursive: bool) -> Result<()> {
        if src.is_dir() {
            if recursive {
                copy_dir_recursive(src, dst)?;
            }
        } else {
            fs::copy(src, dst)?;
        }
        Ok(())
    }

    fn move_entry(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::rename(src, dst)?;
        Ok(())
    }

    fn delete(&self, path: &Path, recursive: bool) -> Result<()> {
        if path.is_dir() {
            if recursive {
                fs::remove_dir_all(path)?;
            }
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn mkdir(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        Ok(())
    }

    fn rename(&self, path: &Path, new_name: &str) -> Result<()> {
        let mut new_path = path.parent().unwrap_or_else(|| Path::new("/")).to_path_buf();
        new_path.push(new_name);
        fs::rename(path, new_path)?;
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

use chrono::DateTime;
