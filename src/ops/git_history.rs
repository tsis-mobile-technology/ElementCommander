use anyhow::Result;
use std::path::Path;
use std::process::Command;
use std::str;

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub date: String,
    pub author: String,
    pub message: String,
}

pub struct GitReport {
    pub root: String,
    pub branch: String,
    pub total_commits: usize,
    pub last_commit_date: String,
    pub recent_commits: Vec<CommitInfo>,
    pub top_authors: Vec<(String, usize)>,
    pub hot_files: Vec<(String, usize)>,
}

pub fn is_git_repo(path: &Path) -> bool {
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
    {
        output.status.success()
    } else {
        false
    }
}

pub fn analyze_git_history(path: &Path) -> Result<GitReport> {
    // 현재 브랜치
    let branch = run_git_command(path, &["branch", "--show-current"])
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    // 총 커밋 수
    let total_commits_str = run_git_command(path, &["rev-list", "--count", "HEAD"])?;
    let total_commits = total_commits_str.trim().parse::<usize>().unwrap_or(0);

    // 마지막 커밋 날짜
    let last_commit_date = run_git_command(path, &["log", "-1", "--format=%ad", "--date=short"])
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    // 최근 커밋 20개
    let recent_commits = parse_commits(path)?;

    // 기여자별 커밋 수
    let top_authors = parse_authors(path)?;

    // 자주 변경된 파일
    let hot_files = parse_hot_files(path)?;

    Ok(GitReport {
        root: path.display().to_string(),
        branch,
        total_commits,
        last_commit_date,
        recent_commits,
        top_authors,
        hot_files,
    })
}

fn run_git_command(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!("Git command failed"))
    }
}

fn parse_commits(path: &Path) -> Result<Vec<CommitInfo>> {
    let output = run_git_command(
        path,
        &["log", "-20", "--pretty=format:%h|%ad|%an|%s", "--date=short"],
    )?;

    let mut commits = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            commits.push(CommitInfo {
                hash: parts[0].to_string(),
                date: parts[1].to_string(),
                author: parts[2].to_string(),
                message: parts[3].to_string(),
            });
        }
    }
    Ok(commits)
}

fn parse_authors(path: &Path) -> Result<Vec<(String, usize)>> {
    let output = run_git_command(path, &["shortlog", "-sn", "--no-merges"])?;

    let mut authors = Vec::new();
    for line in output.lines().take(10) {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(count) = parts[0].parse::<usize>() {
                let name = parts[1..].join(" ");
                authors.push((name, count));
            }
        }
    }
    Ok(authors)
}

fn parse_hot_files(path: &Path) -> Result<Vec<(String, usize)>> {
    // git log --name-only --pretty=format:"" 로 변경된 모든 파일 나열
    let output = run_git_command(path, &["log", "--name-only", "--pretty=format:"])?;

    let mut file_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *file_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }

    let mut hot_files: Vec<(String, usize)> = file_counts.into_iter().collect();
    hot_files.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(hot_files.into_iter().take(10).collect())
}

pub fn format_report(report: &GitReport) -> String {
    let mut result = format!(
        "경로: {}\n브랜치: {}\n총 커밋: {}\n마지막 커밋: {}\n\n",
        report.root, report.branch, report.total_commits, report.last_commit_date
    );

    if !report.recent_commits.is_empty() {
        result.push_str("■ 최근 커밋 (20개)\n");
        for commit in &report.recent_commits {
            result.push_str(&format!(
                "  {} ({}): {} - {}\n",
                commit.hash, commit.date, commit.author, commit.message
            ));
        }
        result.push('\n');
    }

    if !report.top_authors.is_empty() {
        result.push_str("■ 주요 기여자\n");
        for (author, count) in &report.top_authors {
            result.push_str(&format!("  {}: {} 커밋\n", author, count));
        }
        result.push('\n');
    }

    if !report.hot_files.is_empty() {
        result.push_str("■ 자주 변경된 파일 (Top 10)\n");
        for (file, count) in &report.hot_files {
            result.push_str(&format!("  {} ({} 번)\n", file, count));
        }
    }

    result
}
