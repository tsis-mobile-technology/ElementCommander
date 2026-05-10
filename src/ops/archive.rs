use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Write, Read};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

pub fn pack_files(srcs: &[PathBuf], dst_path: &Path) -> Result<()> {
    let extension = dst_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "zip" => pack_zip(srcs, dst_path),
        "tar" => pack_tar(srcs, dst_path),
        "gz" | "tgz" => pack_tar_gz(srcs, dst_path),
        _ => Err(anyhow!("Unsupported archive format for packing: {}", extension)),
    }
}

fn pack_zip(srcs: &[PathBuf], dst_path: &Path) -> Result<()> {
    let file = File::create(dst_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for src in srcs {
        if src.is_dir() {
            let base_path = src.parent().unwrap_or(Path::new(""));
            for entry in WalkDir::new(src) {
                let entry = entry?;
                let path = entry.path();
                let name = path.strip_prefix(base_path)?;
                let name_str = name.to_string_lossy().to_string();

                if path.is_dir() {
                    zip.add_directory(name_str, options)?;
                } else {
                    zip.start_file(name_str, options)?;
                    let mut f = File::open(path)?;
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;
                }
            }
        } else {
            let name = src.file_name().ok_or_else(|| anyhow!("Invalid source path"))?;
            zip.start_file(name.to_string_lossy().to_string(), options)?;
            let mut f = File::open(src)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn pack_tar(srcs: &[PathBuf], dst_path: &Path) -> Result<()> {
    let file = File::create(dst_path)?;
    let mut tar = tar::Builder::new(file);

    for src in srcs {
        if src.is_dir() {
            let base_path = src.parent().unwrap_or(Path::new(""));
            for entry in WalkDir::new(src) {
                let entry = entry?;
                let path = entry.path();
                let name = path.strip_prefix(base_path)?;
                
                if path.is_dir() {
                    tar.append_dir(name, path)?;
                } else {
                    let mut f = File::open(path)?;
                    tar.append_file(name, &mut f)?;
                }
            }
        } else {
            let name = src.file_name().ok_or_else(|| anyhow!("Invalid source path"))?;
            let mut f = File::open(src)?;
            tar.append_file(name, &mut f)?;
        }
    }

    tar.finish()?;
    Ok(())
}

fn pack_tar_gz(srcs: &[PathBuf], dst_path: &Path) -> Result<()> {
    let file = File::create(dst_path)?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    for src in srcs {
        if src.is_dir() {
            let base_path = src.parent().unwrap_or(Path::new(""));
            for entry in WalkDir::new(src) {
                let entry = entry?;
                let path = entry.path();
                let name = path.strip_prefix(base_path)?;
                
                if path.is_dir() {
                    tar.append_dir(name, path)?;
                } else {
                    let mut f = File::open(path)?;
                    tar.append_file(name, &mut f)?;
                }
            }
        } else {
            let name = src.file_name().ok_or_else(|| anyhow!("Invalid source path"))?;
            let mut f = File::open(src)?;
            tar.append_file(name, &mut f)?;
        }
    }

    tar.finish()?;
    Ok(())
}
