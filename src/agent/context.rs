use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};

use crate::linux_terminal::{
    git::ops,
    persist::{PaneFocus, SessionSnapshot, WorkspaceSnapshot},
};

use super::{data_root, ensure_parent, memory, patch_session_path};

const DEFAULT_TERMINAL_LINES: usize = 80;
const TOKEN_LIMIT: usize = 8_000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct WorkspaceContext {
    pub(crate) terminal: TerminalContext,
    pub(crate) git: GitContext,
    pub(crate) logs: LogContext,
    pub(crate) active_pane: ActivePaneContext,
    pub(crate) annotations: Vec<PatchAnnotation>,
    pub(crate) memory: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct TerminalContext {
    pub(crate) cwd: Option<String>,
    pub(crate) last_lines: Vec<String>,
    pub(crate) last_exit_code: Option<i32>,
    pub(crate) last_command: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct GitContext {
    pub(crate) repo_root: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) upstream: Option<String>,
    pub(crate) ahead: u32,
    pub(crate) behind: u32,
    pub(crate) staged_summary: Vec<String>,
    pub(crate) unstaged_summary: Vec<String>,
    pub(crate) conflicted: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct LogContext {
    pub(crate) last_entries: Vec<LogSnapshot>,
    pub(crate) level_distribution: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct LogSnapshot {
    pub(crate) level: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ActivePaneContext {
    pub(crate) tab_title: Option<String>,
    pub(crate) split_focus: Option<String>,
    pub(crate) side_pane: Option<String>,
    pub(crate) strip_mode: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct PatchAnnotation {
    pub(crate) file: String,
    pub(crate) hunk_index: usize,
    pub(crate) status: String,
    pub(crate) annotation: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct UiRuntimeState {
    pub(crate) side_pane: Option<String>,
    pub(crate) strip_mode: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct LogRuntimeSnapshot {
    pub(crate) last_entries: Vec<LogSnapshot>,
    pub(crate) level_distribution: BTreeMap<String, usize>,
}

pub(crate) fn build_workspace_context() -> WorkspaceContext {
    let workspace = load_workspace_snapshot().unwrap_or_default();
    let active_session = active_session(&workspace);
    let terminal = terminal_context(active_session.as_ref());
    let git = git_context(terminal.cwd.as_deref());
    WorkspaceContext {
        memory: git
            .repo_root
            .as_deref()
            .zip(git.branch.as_deref())
            .map(|(root, branch)| memory::load_recent_summaries(Path::new(root), branch, 3))
            .unwrap_or_default(),
        active_pane: active_pane_context(&workspace),
        annotations: patch_annotations(git.repo_root.as_deref(), git.branch.as_deref()),
        logs: load_log_snapshot(),
        terminal,
        git,
    }
}

pub(crate) fn token_budgeted_json(context: &WorkspaceContext, max_tokens: usize) -> String {
    let mut reduced = context.clone();
    let budget = max_tokens.min(TOKEN_LIMIT);
    loop {
        let json = serde_json::to_string(&reduced).unwrap_or_else(|_| "{}".to_string());
        if approx_tokens(&json) <= budget {
            return json;
        }
        if shrink_context(&mut reduced) {
            // Fully shrunk but still over budget — return what we have.
            return serde_json::to_string(&reduced).unwrap_or_else(|_| "{}".to_string());
        }
    }
}

pub(crate) fn write_ui_runtime_state(state: &UiRuntimeState) {
    let path = data_root().join("agent/ui_state.json");
    ensure_parent(&path);
    if let Ok(json) = serde_json::to_string(state) {
        let _ = fs::write(path, json);
    }
}

pub(crate) fn load_ui_runtime_state() -> UiRuntimeState {
    fs::read_to_string(data_root().join("agent/ui_state.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<UiRuntimeState>(&raw).ok())
        .unwrap_or_default()
}

pub(crate) fn write_log_runtime_snapshot(snapshot: &LogRuntimeSnapshot) {
    let path = data_root().join("agent/logr_snapshot.json");
    ensure_parent(&path);
    if let Ok(json) = serde_json::to_string(snapshot) {
        let _ = fs::write(path, json);
    }
}

fn shrink_context(context: &mut WorkspaceContext) -> bool {
    if context.terminal.last_lines.len() > 24 {
        context.terminal.last_lines.drain(0..8);
        return false;
    }
    if context.logs.last_entries.len() > 20 {
        context.logs.last_entries.truncate(20);
        return false;
    }
    if context.annotations.len() > 8 {
        context.annotations.truncate(8);
        return false;
    }
    context.memory.truncate(1);
    true
}

fn approx_tokens(json: &str) -> usize {
    json.chars().count() / 4 + 1
}

fn load_workspace_snapshot() -> Option<WorkspaceSnapshot> {
    crate::linux_terminal::persist::load_workspace().ok().flatten()
}

fn active_session(workspace: &WorkspaceSnapshot) -> Option<SessionSnapshot> {
    let tab = workspace.tabs.get(workspace.active_tab)?;
    let pane = match tab.active_pane {
        PaneFocus::Left => tab.left_pane.as_ref()?,
        PaneFocus::Right => tab.right_pane.as_ref().or(tab.left_pane.as_ref())?,
    };
    pane.sessions.get(pane.active_session).cloned()
}

fn terminal_context(session: Option<&SessionSnapshot>) -> TerminalContext {
    let Some(session) = session else {
        return TerminalContext::default();
    };
    let (last_exit_code, last_command) = read_status(session.status_path.as_deref());
    TerminalContext {
        cwd: session.cwd.clone(),
        last_lines: capture_tmux_lines(session, DEFAULT_TERMINAL_LINES),
        last_exit_code,
        last_command,
    }
}

fn git_context(cwd: Option<&str>) -> GitContext {
    let Some(cwd) = cwd.map(PathBuf::from) else {
        return GitContext::default();
    };
    let Ok(repo_root) = ops::git_repo_root(&cwd) else {
        return GitContext::default();
    };
    let Ok(status) = ops::git_status(&repo_root) else {
        return GitContext::default();
    };
    GitContext {
        repo_root: Some(repo_root.display().to_string()),
        branch: Some(status.branch.clone()),
        upstream: status.upstream,
        ahead: status.ahead,
        behind: status.behind,
        staged_summary: status.staged.iter().map(|item| item.path.clone()).take(24).collect(),
        unstaged_summary: status
            .unstaged
            .iter()
            .map(|item| item.path.clone())
            .chain(status.untracked)
            .take(24)
            .collect(),
        conflicted: status.conflicted,
    }
}

fn active_pane_context(workspace: &WorkspaceSnapshot) -> ActivePaneContext {
    let runtime = load_ui_runtime_state();
    ActivePaneContext {
        tab_title: workspace.tabs.get(workspace.active_tab).map(|tab| tab.title.clone()),
        split_focus: workspace.tabs.get(workspace.active_tab).map(|tab| match tab.active_pane {
            PaneFocus::Left => "left".to_string(),
            PaneFocus::Right => "right".to_string(),
        }),
        side_pane: runtime.side_pane,
        strip_mode: runtime.strip_mode,
    }
}

fn load_log_snapshot() -> LogContext {
    fs::read_to_string(data_root().join("agent/logr_snapshot.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<LogRuntimeSnapshot>(&raw).ok())
        .map(|snapshot| LogContext {
            last_entries: snapshot.last_entries,
            level_distribution: snapshot.level_distribution,
        })
        .unwrap_or_default()
}

fn patch_annotations(repo_root: Option<&str>, branch: Option<&str>) -> Vec<PatchAnnotation> {
    let (Some(root), Some(branch)) = (repo_root, branch) else {
        return Vec::new();
    };
    let path = patch_session_path(Path::new(root), branch);
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    json.get("hunks")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| Some(PatchAnnotation {
            file: item.get("file")?.as_str()?.to_string(),
            hunk_index: item.get("hunk_index")?.as_u64()? as usize,
            status: item.get("status").and_then(serde_json::Value::as_str).unwrap_or("unknown").to_string(),
            annotation: item.get("annotation").and_then(serde_json::Value::as_str).unwrap_or("").to_string(),
        }))
        .take(24)
        .collect()
}


fn read_status(path: Option<&str>) -> (Option<i32>, Option<String>) {
    let Some(path) = path else {
        return (None, None);
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return (None, None);
    };
    let mut parts = raw.trim().splitn(3, '\t');
    let _ = parts.next();
    let status = parts.next().and_then(|value| value.parse::<i32>().ok());
    let command = parts.next().map(ToString::to_string);
    (status, command)
}

fn capture_tmux_lines(session: &SessionSnapshot, lines: usize) -> Vec<String> {
    let (Some(socket), Some(session_id)) = (&session.socket_path, &session.session_id) else {
        return Vec::new();
    };
    let Ok(output) = Command::new("tmux")
        .args(["-S", socket, "capture-pane", "-p", "-t", session_id, "-J", &format!("-S-{lines}")])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_string())
        .collect()
}
