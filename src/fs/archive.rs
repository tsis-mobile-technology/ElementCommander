use super::{FileEntry, FileSystem};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;
use zip::ZipArchive;
use chrono::{Local, TimeZone};

pub struct ArchiveFs {
    archive_path: PathBuf,
    entries: Vec<FileEntry>,
}

impl ArchiveFs {
    pub fn new(path: PathBuf) -> Result<Self> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let entries = match extension.as_str() {
            "zip" => Self::list_zip(&path)?,
            "tar" => Self::list_tar(&path)?,
            "gz" | "tgz" => Self::list_tar_gz(&path)?,
            _ => return Err(anyhow!("Unsupported archive format")),
        };

        Ok(ArchiveFs {
            archive_path: path,
            entries,
        })
    }

    fn extract_zip(&self, src: &Path, dst: &Path) -> Result<()> {
        let file = File::open(&self.archive_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        
        // Find the entry in zip. src is the relative path in zip.
        let src_str = src.to_string_lossy();
        
        // If it's a directory, we need to extract all files starting with this path
        let mut found = false;
        
        // We need to clone names because ZipArchive needs mutable access
        let names: Vec<String> = archive.file_names().map(|n| n.to_string()).collect();
        
        for name in names {
            if name == src_str.as_ref() || name.starts_with(&format!("{}/", src_str)) {
                found = true;
                let mut file = archive.by_name(&name)?;
                let relative_path = Path::new(&name).strip_prefix(src).unwrap_or(Path::new(""));
                let target_path = dst.join(relative_path);
                
                if file.is_dir() {
                    std::fs::create_dir_all(&target_path)?;
                } else {
                    if let Some(parent) = target_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = File::create(&target_path)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }
        }
        
        if !found {
            return Err(anyhow!("File not found in archive: {:?}", src));
        }
        
        Ok(())
    }

    fn extract_tar(&self, src: &Path, dst: &Path) -> Result<()> {
        let file = File::open(&self.archive_path)?;
        let mut archive = tar::Archive::new(file);
        
        let src_str = src.to_string_lossy();
        let mut found = false;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let path_str = path.to_string_lossy();
            
            if path_str == src_str || path_str.starts_with(&format!("{}/", src_str)) {
                found = true;
                let relative_path = path.strip_prefix(src).unwrap_or(Path::new(""));
                let target_path = dst.join(relative_path);
                entry.unpack(target_path)?;
            }
        }

        if !found {
            return Err(anyhow!("File not found in archive: {:?}", src));
        }
        
        Ok(())
    }

    fn extract_tar_gz(&self, src: &Path, dst: &Path) -> Result<()> {
        let file = File::open(&self.archive_path)?;
        let tar_gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar_gz);
        
        let src_str = src.to_string_lossy();
        let mut found = false;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let path_str = path.to_string_lossy();
            
            if path_str == src_str || path_str.starts_with(&format!("{}/", src_str)) {
                found = true;
                let relative_path = path.strip_prefix(src).unwrap_or(Path::new(""));
                let target_path = dst.join(relative_path);
                entry.unpack(target_path)?;
            }
        }

        if !found {
            return Err(anyhow!("File not found in archive: {:?}", src));
        }
        
        Ok(())
    }

    fn list_zip(path: &Path) -> Result<Vec<FileEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        let mut entries = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();
            let is_dir = file.is_dir();
            let size = file.size();
            
            // Convert ZIP time to chrono::DateTime
            let last_mod = file.last_modified();
            let modified = Local.with_ymd_and_hms(
                last_mod.year() as i32,
                last_mod.month() as u32,
                last_mod.day() as u32,
                last_mod.hour() as u32,
                last_mod.minute() as u32,
                last_mod.second() as u32,
            ).single().unwrap_or_else(Local::now);

            entries.push(FileEntry {
                name,
                path: PathBuf::from(file.name()),
                is_dir,
                size,
                modified,
                permissions: file.unix_mode().unwrap_or(0),
            });
        }

        Ok(entries)
    }

    fn list_tar(path: &Path) -> Result<Vec<FileEntry>> {
        let file = File::open(path)?;
        let mut archive = tar::Archive::new(file);
        let mut entries = Vec::new();

        for entry in archive.entries()? {
            let entry = entry?;
            let header = entry.header();
            let path = entry.path()?.to_path_buf();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let is_dir = entry.header().entry_type().is_dir();
            let size = entry.header().size()?;
            
            let mtime = header.mtime()?;
            let modified = Local.timestamp_opt(mtime as i64, 0).single().unwrap_or_else(Local::now);

            entries.push(FileEntry {
                name,
                path,
                is_dir,
                size,
                modified,
                permissions: header.mode()?,
            });
        }

        Ok(entries)
    }

    fn list_tar_gz(path: &Path) -> Result<Vec<FileEntry>> {
        let file = File::open(path)?;
        let tar_gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar_gz);
        let mut entries = Vec::new();

        for entry in archive.entries()? {
            let entry = entry?;
            let header = entry.header();
            let path = entry.path()?.to_path_buf();
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let is_dir = entry.header().entry_type().is_dir();
            let size = entry.header().size()?;
            
            let mtime = header.mtime()?;
            let modified = Local.timestamp_opt(mtime as i64, 0).single().unwrap_or_else(Local::now);

            entries.push(FileEntry {
                name,
                path,
                is_dir,
                size,
                modified,
                permissions: header.mode()?,
            });
        }

        Ok(entries)
    }
}

impl FileSystem for ArchiveFs {
    fn list(&self, path: &Path) -> Result<Vec<FileEntry>> {
        let target_path = if path == Path::new("") || path == Path::new("/") {
            Path::new("")
        } else {
            path
        };

        let mut result = Vec::new();
        let mut seen_dirs = std::collections::HashSet::new();

        for entry in &self.entries {
            let entry_path = &entry.path;
            
            // Check if entry is directly inside target_path
            if let Ok(relative) = entry_path.strip_prefix(target_path) {
                let parts: Vec<_> = relative.components().collect();
                
                if parts.is_empty() {
                    continue;
                }

                let first_part = parts[0].as_os_str().to_string_lossy().to_string();
                if first_part.is_empty() {
                    continue;
                }

                if parts.len() == 1 {
                    // Direct file or directory
                    let mut e = entry.clone();
                    e.name = first_part.clone();
                    if e.is_dir {
                        seen_dirs.insert(first_part);
                    }
                    result.push(e);
                } else if parts.len() > 1 {
                    // Entry is deeper, add the immediate directory if not already added
                    if !seen_dirs.contains(&first_part) {
                        seen_dirs.insert(first_part.clone());
                        result.push(FileEntry {
                            name: first_part.clone(),
                            path: target_path.join(&first_part),
                            is_dir: true,
                            size: 0,
                            modified: entry.modified,
                            permissions: 0,
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    fn copy(&self, src: &Path, dst: &Path, _recursive: bool) -> Result<()> {
        let extension = self.archive_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "zip" => self.extract_zip(src, dst),
            "tar" => self.extract_tar(src, dst),
            "gz" | "tgz" => self.extract_tar_gz(src, dst),
            _ => Err(anyhow!("Unsupported archive format")),
        }
    }

    fn move_entry(&self, _src: &Path, _dst: &Path) -> Result<()> {
        Err(anyhow!("Moving within archives not supported yet"))
    }

    fn delete(&self, _path: &Path, _recursive: bool) -> Result<()> {
        Err(anyhow!("Deleting from archives not supported yet"))
    }

    fn mkdir(&self, _path: &Path) -> Result<()> {
        Err(anyhow!("Creating directories in archives not supported yet"))
    }

    fn rename(&self, _path: &Path, _new_name: &str) -> Result<()> {
        Err(anyhow!("Renaming in archives not supported yet"))
    }

    fn exists(&self, path: &Path) -> bool {
        self.entries.iter().any(|e| e.path == path || e.path.starts_with(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        if path == Path::new("") || path == Path::new("/") {
            return true;
        }
        self.entries.iter().any(|e| e.path == path && e.is_dir) ||
        self.entries.iter().any(|e| e.path.starts_with(path) && e.path != path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    #[test]
    fn test_zip_listing() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        
        // Create a test zip file
        {
            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            
            zip.start_file("file1.txt", FileOptions::default()).unwrap();
            zip.write_all(b"content1").unwrap();
            
            zip.add_directory("dir1/", FileOptions::default()).unwrap();
            zip.start_file("dir1/file2.txt", FileOptions::default()).unwrap();
            zip.write_all(b"content2").unwrap();
            
            zip.finish().unwrap();
        }

        let fs = ArchiveFs::new(zip_path).unwrap();
        
        // Root listing
        let entries = fs.list(Path::new("")).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "file1.txt"));
        assert!(entries.iter().any(|e| e.name == "dir1"));

        // Dir1 listing
        let entries = fs.list(Path::new("dir1")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file2.txt");
    }
}
