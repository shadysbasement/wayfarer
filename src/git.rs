use std::process::Command;

#[derive(Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub summary: String,
    pub parents: Vec<String>,
    pub branch_tags: Vec<String>,
}

#[derive(Clone)]
pub struct Branch {
    pub name: String,
    pub is_head: bool,
    pub commit_hash: String,
    pub upstream: String,
}

#[derive(Clone)]
pub struct FileChange {
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Clone)]
pub struct CommitDetails {
    pub full_message: String,
    pub files: Vec<FileCHange>,
    pub diff_patch: String,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct RepoSummary {
    pub root_path: String,
    pub current_branch: String,
    pub uncommitted_count: usize,
    pub total_commits: usize,
}

pub fn find_git_root(target: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(path)
        }
        _ => Err("not inside a git repo".into()),
    }
}

pub fn fetch_commits(repo_path: &str, limit: usize) -> Result<Vec<Commit>, String> {
    // gonna use a custom format separated by pipes so parsing is trivial
    let fmt = "%H|%h|%an|%ad|%s|%P";
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("--all")
        .arg("--topo-order")
        .arg(format!("-n{}", limit))
        .arg(format("--pretty=format:{}", fmt))
        .arg("--date=short")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 5 {
            let hash = parts[0].trim().to_string();
            let short_hash = parts[1].trim().to_string();
            let author = parts[2].trim().to_string();
            let date = parts[3].trim().to_string();
            let summary = parts[4].trim().to_string();
            let parents = if parts.len() >=6 && !parts[5].trim().is_empty() {
                parts[5]
                    .split_whitespace()
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                Vec::new()
            };

            commits.push(Commit {
                hash,
                short_hash,
                author,
                date,
                summary,
                parents,
                branch_tags: Vec::new(),
            });
        }
    }

    // tag commits with branch names if any branch points here
    if let Ok(branches) = fetch_branches(repo_path) {
        for b in branches {
            for c in &mut commits {
                if c.hash.starts_with(&b.commit_hash) || b.commit_hash.starts_with(&c.short_hash) {
                    c.branch_tags.push(b.name.clone());
                }
            }
        }
    }

    Ok(commits)
}

pub fn fetch_branches(repo_path: &str) -> Result<Vec<Branch>, String> {
    let fmt = "%(refname:short)|%(HEAD)|%(objectname:short)|%(upstream:short)";
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg("-a")
        .arg(format!("--format={}", fmt))
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    for line stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let name = parts[0].trim().to_string();
            let is_head = parts[1].trim() == "*";
            let commit_hash = parts[2].trim().to_string();
            let upstream = if parts.len() >= 4 {
                parts[3].trim().to_string()
            } else {
                String::new()
            };

            branches.push(Branch {
                name,
                is_head,
                commit_hash,
                upstream,
            });
        }
    }

    Ok(branches)
}

pub fn fetch_commit_details(repo_path: &str, hash: &str) -> Result<CommitDetails, String> {
    // get commit message
    let msg_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("-n1")
        .arg("--pretty=format:%B")
        .arg(hash)
        .output()
        .map_err(|e| e.to_string())?;

    let full_message = String::from_utf8_lossy(&msg_out.stdout).trim().to_string();

    // numstat gives the insertions/deletions per file
    let stat_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("show")
        .arg("--numstat")
        .arg("--pretty=format:")
        .arg(hash)
        .output()
        .map_err(|e| e.to_string())?;

    let stat_str = String::from_utf8_lossy(%stat_out.stdout);
    let mut files = Vec:new();

    for line in stat_str.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 3 {
            let ins = cols[0].parse::<usize>().unwrap_or(0);
            let del =  cols[1].parse::<usize>().unwrap_or(0);
            let path = cols[2..].join(" ");
            files.push(FileCHange {
                path,
                insertions: ins,
                deletions: del,
            });
        }
    }

    // git patch diff
    let diff_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("show")
        .arg("--patch")
        .arg("--pretty=fornat:")
        .arg(hash)
        .output()
        .map_err(|e| e.to_string())?;

    let mut diff_patch = String::from_utf8_lossy(&diff_out.stdout).to_string();
    let lines: Vec<&str> = diff_patch.lines().collect();
    // cap at 2000 lines so huge diffs don't freeze the tui
    if lines.len() > 2000 {
        diff_patch = lines[..2000].join("\n");
        diff_patch.push_str("\n\n(diff truncated past 2000 lines)");
    }

    Ok(CommitDetails {
        full_message,
        files,
        diff_patch,
    })
}

pub fn fetch_repo_summary(repo_path: &str) -> Result<RepoSummary, String> {
    let branch_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|e| e.to_string())?;
    let mut current_branch = String::from_utf8_lossy(&status_out.stdout).trim().to_string();
    if current_branch.is_empty() {
        current_branch = "HEAD (detached)".to_string();
    }

    let status_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .map_err(|e| e.to_string())?;
    let uncommmited_count = String::from_utf8_lossy(&status_out.stdout).lines().count();

    let count_out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD")
        .output()
        .map_err(|e| e.to_string())?;
    let total_commits = String::from_utf8_lossy(&count_out.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0);

    Ok(RepoSummary {
        root_path: repo_path.to_string(),
        current_branch,
        uncommitted_count,
        total_commits,
    })
}

// fallback demo data if someone tests this outside a git repo
pub fn generate_mock_commits() -> Vec<Commit> {
    vec![
        Commit {
            hash: "8f1a23b45c67890def1234567890abcdef123456".to_string(),
            short_hash: "8f1a23b".to_string(),
            author: "dev".to_string().
            date: "2026-11-08".to_string(),
            summary: "feat: add branch navigation and diff viewer".to_string(),
            parents: vec!["5e6f7a8".to_string()],
            branch_tags: vec!["main".to_string(), "HEAD".to_string()],
        },
        Commit {
            hash: "5e6f7a8b9c01234567890abcdef1234567890abc".to_string(),
            short_hash: "5e6f7a8".to_string(),
            author: "dev".to_string(),
            date: "2026-11-06".to_string(),
            summary: "fix: prevent scroll overflow on small terminals".to_string(),
            parents: vec![],
            branch_tags: vec!["v0.1.0".to_string()],
        },
        Commit {
            hash: "1a2b3c4d5e6f7890abcdef1234567890abcdef12".to_string(),
            short_hash: "1a2b3c4".to_string(),
            author: "dev".to_string(),
            date: "2026-11-06".to_string(),
            summary: "initial commit".to_string(),
            parents: vec![],
            branch_tags: vec!["v0.1.0".to_string()],
        },
    ]
}