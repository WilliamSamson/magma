use std::{
    path::{Path, PathBuf},
    process::Command,
};

// ─── Data types ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct RepoStatus {
    pub branch: String,
    pub is_detached: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
    pub has_remotes: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct FileStatus {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileChange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileChange {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Conflict,
    Unknown,
}

#[derive(Clone, Debug)]
pub(super) struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub refs: String,
    pub graph_line: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug)]
pub(super) struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiffLineKind {
    Added,
    Removed,
    Context,
    Header,
}

#[derive(Clone, Debug)]
pub(super) struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
    pub last_commit: String,
}

#[derive(Clone, Debug)]
pub(super) struct StashEntry {
    pub index: usize,
    pub message: String,
}

#[derive(Clone, Debug)]
pub(super) struct BlameLine {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub line_number: usize,
    pub content: String,
}

// ─── Git command runner ──────────────────────────────────────────────

fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let msg = stderr.trim();
        Err(if msg.is_empty() {
            format!("git exited with {}", output.status)
        } else {
            msg.to_string()
        })
    }
}

// ─── Repository detection ────────────────────────────────────────────

pub(crate) fn git_repo_root(cwd: &Path) -> Result<PathBuf, String> {
    let out = run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(out.trim()))
}

// ─── Status ──────────────────────────────────────────────────────────

pub(crate) fn git_status(root: &Path) -> Result<RepoStatus, String> {
    let out = run_git(root, &["status", "--porcelain=v2", "--branch"])?;

    let mut branch = String::from("(unknown)");
    let mut is_detached = false;
    let mut upstream = None;
    let mut ahead: u32 = 0;
    let mut behind: u32 = 0;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicted = Vec::new();

    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            if rest == "(detached)" {
                is_detached = true;
                // Get short hash for detached HEAD display
                branch = git_short_head(root).unwrap_or_else(|_| "detached".to_string());
            } else {
                branch = rest.to_string();
            }
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            upstream = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if let Some(a) = parts.first() {
                ahead = a.trim_start_matches('+').parse().unwrap_or(0);
            }
            if let Some(b) = parts.get(1) {
                behind = b.trim_start_matches('-').parse().unwrap_or(0);
            }
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            // Changed entry: "1 XY sub mH mI mW hH hP path"
            // Rename entry:  "2 XY sub mH mI mW hH hP X{score} path\torigpath"
            let is_rename = line.starts_with("2 ");
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() >= 9 {
                let xy = parts[1];
                let path_part = parts[8];
                let (path, old_path) = if is_rename {
                    let mut split = path_part.splitn(2, '\t');
                    let new = split.next().unwrap_or(path_part).to_string();
                    let old = split.next().map(|s| s.to_string());
                    (new, old)
                } else {
                    (path_part.to_string(), None)
                };

                let x = xy.chars().next().unwrap_or('.');
                let y = xy.chars().nth(1).unwrap_or('.');

                if x != '.' {
                    staged.push(FileStatus {
                        path: path.clone(),
                        old_path: old_path.clone(),
                        status: parse_change(x),
                    });
                }
                if y != '.' {
                    unstaged.push(FileStatus {
                        path,
                        old_path,
                        status: parse_change(y),
                    });
                }
            }
        } else if line.starts_with("u ") {
            // Unmerged entry: "u XY sub m1 m2 m3 mW h1 h2 h3 path"
            let parts: Vec<&str> = line.splitn(11, ' ').collect();
            if let Some(&path) = parts.last() {
                conflicted.push(path.to_string());
            }
        } else if line.starts_with("? ") {
            if let Some(path) = line.strip_prefix("? ") {
                untracked.push(path.to_string());
            }
        }
    }

    let has_remotes = has_any_remote(root);

    Ok(RepoStatus {
        branch,
        is_detached,
        upstream,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        conflicted,
        has_remotes,
    })
}

fn git_short_head(root: &Path) -> Result<String, String> {
    let out = run_git(root, &["rev-parse", "--short", "HEAD"])?;
    Ok(out.trim().to_string())
}

fn has_any_remote(root: &Path) -> bool {
    run_git(root, &["remote"])
        .map(|out| !out.trim().is_empty())
        .unwrap_or(false)
}

fn parse_change(c: char) -> FileChange {
    match c {
        'A' => FileChange::Added,
        'M' => FileChange::Modified,
        'D' => FileChange::Deleted,
        'R' => FileChange::Renamed,
        'C' => FileChange::Copied,
        'U' => FileChange::Conflict,
        _ => FileChange::Unknown,
    }
}

// ─── Diff ────────────────────────────────────────────────────────────

pub(super) fn git_diff_file(root: &Path, path: &str, staged: bool) -> Result<Vec<DiffHunk>, String> {
    let mut args = vec!["diff", "--color=never", "-U3"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(path);
    let out = run_git(root, &args)?;
    Ok(parse_diff(&out))
}

pub(super) fn git_diff_stat(root: &Path, commit: &str) -> Result<String, String> {
    run_git(root, &["show", "--stat", "--color=never", "--format=", commit])
}

pub(super) fn git_show_diff(root: &Path, commit: &str) -> Result<Vec<DiffHunk>, String> {
    let out = run_git(root, &["show", "--color=never", "-U3", "--format=", commit])?;
    Ok(parse_diff(&out))
}

pub(super) fn parse_diff_text(text: &str) -> Vec<DiffHunk> {
    parse_diff(text)
}

fn parse_diff(text: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_header = String::new();
    let mut current_lines: Vec<DiffLine> = Vec::new();
    let mut in_hunk = false;
    let mut is_binary = false;

    for line in text.lines() {
        // Detect binary files
        if line.starts_with("Binary files") || line.contains("differ") && line.contains("Binary") {
            is_binary = true;
            continue;
        }

        if line.starts_with("@@") {
            if in_hunk && !current_lines.is_empty() {
                hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            current_header = line.to_string();
            in_hunk = true;
        } else if line.starts_with("diff --git") {
            // New file in multi-file diff — flush current hunk
            if in_hunk && !current_lines.is_empty() {
                hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            in_hunk = false;
            is_binary = false;
        } else if line.starts_with("index ") || line.starts_with("---") || line.starts_with("+++") {
            // Skip diff metadata headers
        } else if in_hunk {
            let kind = if line.starts_with('+') {
                DiffLineKind::Added
            } else if line.starts_with('-') {
                DiffLineKind::Removed
            } else {
                DiffLineKind::Context
            };
            current_lines.push(DiffLine {
                kind,
                content: line.to_string(),
            });
        }
    }

    if in_hunk && !current_lines.is_empty() {
        hunks.push(DiffHunk {
            header: current_header,
            lines: current_lines,
        });
    }

    // If the only content was binary, return a marker hunk
    if hunks.is_empty() && is_binary {
        hunks.push(DiffHunk {
            header: String::new(),
            lines: vec![DiffLine {
                kind: DiffLineKind::Context,
                content: "(binary file)".to_string(),
            }],
        });
    }

    hunks
}

// ─── Full diff (all files, unstaged + staged) ────────────────────

/// Returns the raw unified diff for all changed files (both staged and unstaged).
/// Each entry is (file_path, raw_diff_text).
pub(super) fn git_diff_all_hunks(root: &Path) -> Result<Vec<(String, Vec<DiffHunk>)>, String> {
    let mut result: Vec<(String, Vec<DiffHunk>)> = Vec::new();

    // Unstaged changes
    let unstaged = run_git(root, &["diff", "--color=never", "-U3"])?;
    collect_file_hunks(&unstaged, &mut result);

    // Staged changes
    let staged = run_git(root, &["diff", "--cached", "--color=never", "-U3"])?;
    collect_file_hunks(&staged, &mut result);

    Ok(result)
}

/// Parse a multi-file diff and group hunks by file path.
fn collect_file_hunks(diff_text: &str, result: &mut Vec<(String, Vec<DiffHunk>)>) {
    let mut current_file = String::new();
    let mut current_header = String::new();
    let mut current_lines: Vec<DiffLine> = Vec::new();
    let mut file_hunks: Vec<DiffHunk> = Vec::new();
    let mut in_hunk = false;

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            // Flush previous hunk
            if in_hunk && !current_lines.is_empty() {
                file_hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            // Flush previous file
            if !current_file.is_empty() && !file_hunks.is_empty() {
                // Merge into existing file entry or create new
                if let Some(entry) = result.iter_mut().find(|(f, _)| f == &current_file) {
                    entry.1.append(&mut file_hunks);
                } else {
                    result.push((current_file.clone(), std::mem::take(&mut file_hunks)));
                }
                file_hunks.clear();
            }
            // Extract file path from "diff --git a/path b/path"
            current_file = line
                .strip_prefix("diff --git a/")
                .and_then(|rest| rest.split(" b/").next())
                .unwrap_or("")
                .to_string();
            in_hunk = false;
        } else if line.starts_with("@@") {
            if in_hunk && !current_lines.is_empty() {
                file_hunks.push(DiffHunk {
                    header: current_header.clone(),
                    lines: std::mem::take(&mut current_lines),
                });
            }
            current_header = line.to_string();
            in_hunk = true;
        } else if line.starts_with("index ") || line.starts_with("---") || line.starts_with("+++") {
            // Skip metadata
        } else if line.starts_with("Binary files") {
            file_hunks.push(DiffHunk {
                header: String::new(),
                lines: vec![DiffLine {
                    kind: DiffLineKind::Context,
                    content: "(binary file)".to_string(),
                }],
            });
        } else if in_hunk {
            let kind = if line.starts_with('+') {
                DiffLineKind::Added
            } else if line.starts_with('-') {
                DiffLineKind::Removed
            } else {
                DiffLineKind::Context
            };
            current_lines.push(DiffLine {
                kind,
                content: line.to_string(),
            });
        }
    }

    // Flush last hunk + file
    if in_hunk && !current_lines.is_empty() {
        file_hunks.push(DiffHunk {
            header: current_header,
            lines: current_lines,
        });
    }
    if !current_file.is_empty() && !file_hunks.is_empty() {
        if let Some(entry) = result.iter_mut().find(|(f, _)| f == &current_file) {
            entry.1.append(&mut file_hunks);
        } else {
            result.push((current_file, file_hunks));
        }
    }
}

// ─── Staging ─────────────────────────────────────────────────────────

pub(super) fn git_stage_file(root: &Path, path: &str) -> Result<(), String> {
    run_git(root, &["add", "--", path])?;
    Ok(())
}

pub(super) fn git_unstage_file(root: &Path, path: &str) -> Result<(), String> {
    run_git(root, &["restore", "--staged", "--", path])?;
    Ok(())
}

pub(super) fn git_discard_file(root: &Path, path: &str) -> Result<(), String> {
    run_git(root, &["checkout", "--", path])?;
    Ok(())
}

pub(super) fn git_stage_all(root: &Path) -> Result<(), String> {
    run_git(root, &["add", "-A"])?;
    Ok(())
}

pub(super) fn git_unstage_all(root: &Path) -> Result<(), String> {
    run_git(root, &["reset", "HEAD"])?;
    Ok(())
}

// ─── Commit ──────────────────────────────────────────────────────────

pub(super) fn git_commit(root: &Path, message: &str) -> Result<String, String> {
    run_git(root, &["commit", "-m", message])
}

// ─── Log / Graph ─────────────────────────────────────────────────────

pub(super) fn git_log(root: &Path, limit: usize, skip: usize) -> Result<Vec<CommitInfo>, String> {
    let limit_arg = format!("-{limit}");
    let skip_arg = format!("--skip={skip}");
    let out = run_git(
        root,
        &[
            "log",
            &limit_arg,
            &skip_arg,
            "--all",
            "--format=%H%n%h%n%an%n%ar%n%s%n%D%n---END---",
        ],
    )?;
    Ok(parse_log_output(&out))
}

pub(super) fn git_log_graph(root: &Path, limit: usize) -> Result<Vec<CommitInfo>, String> {
    let limit_arg = format!("-{limit}");
    let out = run_git(
        root,
        &[
            "log",
            &limit_arg,
            "--all",
            "--graph",
            "--format=%H%n%h%n%an%n%ar%n%s%n%D%n---END---",
        ],
    )?;
    Ok(parse_graph_output(&out))
}

/// Check if the repo has any commits at all.
pub(super) fn has_commits(root: &Path) -> bool {
    run_git(root, &["rev-parse", "HEAD"]).is_ok()
}

fn parse_log_output(text: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();
    let mut lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        if line.trim() == "---END---" {
            if lines.len() >= 5 {
                commits.push(CommitInfo {
                    hash: lines[0].to_string(),
                    short_hash: lines[1].to_string(),
                    author: lines[2].to_string(),
                    date: lines[3].to_string(),
                    message: lines[4].to_string(),
                    refs: lines.get(5).unwrap_or(&"").to_string(),
                    graph_line: None,
                });
            }
            lines.clear();
        } else {
            lines.push(line);
        }
    }

    commits
}

fn parse_graph_output(text: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();
    let mut field_lines: Vec<String> = Vec::new();
    let mut graph_prefix = String::new();

    for line in text.lines() {
        let trimmed = line.trim_end();

        if trimmed.ends_with("---END---") {
            if field_lines.len() >= 5 {
                commits.push(CommitInfo {
                    hash: field_lines[0].clone(),
                    short_hash: field_lines[1].clone(),
                    author: field_lines[2].clone(),
                    date: field_lines[3].clone(),
                    message: field_lines[4].clone(),
                    refs: field_lines.get(5).map(|s| s.as_str()).unwrap_or("").to_string(),
                    graph_line: Some(graph_prefix.clone()),
                });
            }
            field_lines.clear();
            graph_prefix.clear();
        } else {
            // Separate graph decoration from content
            // Graph chars: * | / \ space _
            let content_start = trimmed
                .find(|c: char| !matches!(c, '*' | '|' | '/' | '\\' | ' ' | '_'))
                .unwrap_or(trimmed.len());

            let graph_part = &trimmed[..content_start];
            let content_part = trimmed[content_start..].to_string();

            if field_lines.is_empty() {
                graph_prefix = graph_part.to_string();
            }

            field_lines.push(content_part);
        }
    }

    commits
}

// ─── Branches ────────────────────────────────────────────────────────

pub(super) fn git_branches(root: &Path) -> Result<(Vec<BranchInfo>, Vec<BranchInfo>), String> {
    // Local branches (separate command — no heuristics needed)
    let local_out = run_git(
        root,
        &[
            "branch",
            "--format=%(HEAD)%(refname:short)\t%(upstream:short)\t%(subject)",
        ],
    )?;

    let mut local = Vec::new();
    for line in local_out.lines() {
        let is_current = line.starts_with('*');
        let line = line.trim_start_matches('*').trim_start_matches(' ');
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let upstream = parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string());
        let last_commit = parts.get(2).unwrap_or(&"").to_string();

        if !name.is_empty() {
            local.push(BranchInfo {
                name,
                is_current,
                upstream,
                last_commit,
            });
        }
    }

    // Remote tracking branches (separate command)
    let remote_out = run_git(
        root,
        &[
            "branch",
            "-r",
            "--format=%(refname:short)\t%(subject)",
        ],
    )?;

    let mut remote = Vec::new();
    for line in remote_out.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let last_commit = parts.get(1).unwrap_or(&"").to_string();

        // Skip the HEAD pointer (e.g. "origin/HEAD")
        if !name.is_empty() && !name.ends_with("/HEAD") {
            remote.push(BranchInfo {
                name,
                is_current: false,
                upstream: None,
                last_commit,
            });
        }
    }

    Ok((local, remote))
}

pub(super) fn git_create_branch(root: &Path, name: &str) -> Result<(), String> {
    // Validate branch name before creating
    run_git(root, &["check-ref-format", "--branch", name])
        .map_err(|_| format!("invalid branch name: {name}"))?;
    run_git(root, &["checkout", "-b", name])?;
    Ok(())
}

pub(super) fn git_switch_branch(root: &Path, name: &str) -> Result<(), String> {
    run_git(root, &["checkout", name])?;
    Ok(())
}

pub(super) fn git_delete_branch(root: &Path, name: &str) -> Result<(), String> {
    run_git(root, &["branch", "-d", name])?;
    Ok(())
}

pub(super) fn git_merge_branch(root: &Path, name: &str) -> Result<String, String> {
    run_git(root, &["merge", name])
}

// ─── Stash ───────────────────────────────────────────────────────────

pub(super) fn git_stash_list(root: &Path) -> Result<Vec<StashEntry>, String> {
    let out = run_git(root, &["stash", "list", "--format=%gd\t%gs"])?;
    let mut entries = Vec::new();

    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() >= 2 {
            let index_str = parts[0]
                .strip_prefix("stash@{")
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or("0");
            let index = index_str.parse().unwrap_or(0);
            entries.push(StashEntry {
                index,
                message: parts[1].to_string(),
            });
        }
    }

    Ok(entries)
}

pub(super) fn git_stash_push(root: &Path, message: Option<&str>) -> Result<(), String> {
    let mut args = vec!["stash", "push"];
    if let Some(msg) = message {
        args.push("-m");
        args.push(msg);
    }
    run_git(root, &args)?;
    Ok(())
}

pub(super) fn git_stash_apply(root: &Path, index: usize) -> Result<String, String> {
    let stash_ref = format!("stash@{{{index}}}");
    run_git(root, &["stash", "apply", &stash_ref])
}

pub(super) fn git_stash_pop(root: &Path, index: usize) -> Result<String, String> {
    let stash_ref = format!("stash@{{{index}}}");
    run_git(root, &["stash", "pop", &stash_ref])
}

pub(super) fn git_stash_drop(root: &Path, index: usize) -> Result<String, String> {
    let stash_ref = format!("stash@{{{index}}}");
    run_git(root, &["stash", "drop", &stash_ref])
}

pub(super) fn git_stash_show(root: &Path, index: usize) -> Result<String, String> {
    let stash_ref = format!("stash@{{{index}}}");
    run_git(root, &["stash", "show", "-p", "--color=never", &stash_ref])
}

// ─── Remote operations ───────────────────────────────────────────────

pub(super) fn git_fetch(root: &Path) -> Result<String, String> {
    run_git(root, &["fetch", "--all", "--prune"])
}

pub(super) fn git_pull(root: &Path) -> Result<String, String> {
    run_git(root, &["pull"])
}

pub(super) fn git_push(root: &Path) -> Result<String, String> {
    run_git(root, &["push"])
}

// ─── Blame ───────────────────────────────────────────────────────────

pub(super) fn git_blame(root: &Path, file: &str) -> Result<Vec<BlameLine>, String> {
    let out = run_git(root, &["blame", "--porcelain", file])?;
    parse_blame_output(&out)
}

fn parse_blame_output(text: &str) -> Result<Vec<BlameLine>, String> {
    let mut lines = Vec::new();
    let mut current_hash = String::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_line_num: usize = 0;

    for line in text.lines() {
        if let Some(content) = line.strip_prefix('\t') {
            lines.push(BlameLine {
                hash: current_hash.clone(),
                author: current_author.clone(),
                date: current_date.clone(),
                line_number: current_line_num,
                content: content.to_string(),
            });
        } else if let Some(rest) = line.strip_prefix("author ") {
            current_author = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("author-time ") {
            current_date = rest.to_string();
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(hash) = parts.first() {
                if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    current_hash = hash[..8].to_string();
                    if let Some(line_num) = parts.get(2) {
                        current_line_num = line_num.parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    Ok(lines)
}

// ─── Search ──────────────────────────────────────────────────────────

pub(super) fn git_search_commits(root: &Path, query: &str, mode: SearchMode) -> Result<Vec<CommitInfo>, String> {
    let format_arg = "--format=%H%n%h%n%an%n%ar%n%s%n%D%n---END---";
    let mut args = vec!["log", "-50", "--all", format_arg];

    let grep_arg;
    let author_arg;

    match mode {
        SearchMode::Message => {
            grep_arg = format!("--grep={query}");
            args.push(&grep_arg);
            args.push("-i");
        }
        SearchMode::Author => {
            author_arg = format!("--author={query}");
            args.push(&author_arg);
            args.push("-i");
        }
        SearchMode::File => {
            args.push("--");
            args.push(query);
        }
    }

    let out = run_git(root, &args)?;
    Ok(parse_log_output(&out))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SearchMode {
    Message,
    Author,
    File,
}

/// Lightweight fingerprint of repo state for change detection.
/// Combines HEAD ref + index file mtime so the watcher can detect
/// commits, checkouts, staging changes, etc. without a full status parse.
pub(super) fn quick_repo_fingerprint(root: &Path) -> Option<String> {
    let head = run_git(root, &["rev-parse", "HEAD"]).ok()?;
    let index = root.join(".git/index");
    let mtime = std::fs::metadata(&index)
        .ok()?
        .modified()
        .ok()?;
    Some(format!("{}:{:?}", head.trim(), mtime))
}

impl FileChange {
    pub fn label(&self) -> &'static str {
        match self {
            FileChange::Added => "A",
            FileChange::Modified => "M",
            FileChange::Deleted => "D",
            FileChange::Renamed => "R",
            FileChange::Copied => "C",
            FileChange::Conflict => "!",
            FileChange::Unknown => "?",
        }
    }
}
