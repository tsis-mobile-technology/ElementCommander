use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_duplicates(root: &Path) -> Result<Vec<Vec<PathBuf>>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    // 1차 필터: 파일 크기로 그룹화
    let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                size_groups.entry(size).or_insert_with(Vec::new).push(path.to_path_buf());
            }
        }
    }

    // 2차 필터: 크기 그룹이 2개 이상인 파일들의 해시 계산
    let mut hash_groups: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();

    for (_size, paths) in size_groups.iter() {
        if paths.len() < 2 {
            continue;
        }

        for path in paths {
            match compute_file_hash(path) {
                Ok(hash) => {
                    hash_groups.entry(hash).or_insert_with(Vec::new).push(path.clone());
                }
                Err(_) => {
                    // 파일을 읽을 수 없으면 무시
                }
            }
        }
    }

    // 최종 결과: 그룹 크기 ≥ 2인 것만 반환 (경로로 정렬)
    let mut result: Vec<Vec<PathBuf>> = hash_groups
        .into_values()
        .filter(|group| group.len() >= 2)
        .map(|mut group| {
            group.sort();
            group
        })
        .collect();

    result.sort_by(|a, b| a[0].cmp(&b[0]));

    Ok(result)
}

fn compute_file_hash(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_vec())
}
