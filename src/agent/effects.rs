use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

use super::{
    actions::{AgentAction, HunkRef},
    context::build_workspace_context,
    patch_session_path,
};

pub(crate) fn execute_side_effect(action: &AgentAction) -> Result<Option<UiEffect>, String> {
    match action {
        AgentAction::SurfaceMessage { message, .. } => Ok(Some(UiEffect::Message(message.clone()))),
        AgentAction::OpenPane { pane, .. } => Ok(Some(UiEffect::OpenPane(*pane))),
        AgentAction::FilterLogr { filter, .. } => {
            write_logr_filter_request(filter)?;
            Ok(Some(UiEffect::OpenPane(super::actions::PaneType::Logr)))
        }
        AgentAction::StageHunk { hunk, .. } => {
            stage_hunk(hunk)?;
            Ok(Some(UiEffect::Message(format!(
                "staged {}#{}",
                hunk.file, hunk.hunk_index
            ))))
        }
        AgentAction::WriteAnnotation { hunk, note, .. } => {
            write_annotation(hunk, note)?;
            Ok(Some(UiEffect::Message(format!(
                "annotated {}#{}",
                hunk.file, hunk.hunk_index
            ))))
        }
        AgentAction::RunCommand { command, .. } => Ok(Some(UiEffect::DispatchTerminalCommand(
            command.clone(),
        ))),
    }
}

#[derive(Clone, Debug)]
pub(crate) enum UiEffect {
    OpenPane(super::actions::PaneType),
    Message(String),
    DispatchTerminalCommand(String),
}

pub(crate) fn logr_filter_request_path() -> PathBuf {
    super::data_root().join("agent/logr_filter.json")
}

pub(crate) fn take_logr_filter_request() -> Option<super::actions::LogFilter> {
    let path = logr_filter_request_path();
    let raw = fs::read_to_string(&path).ok()?;
    let filter = serde_json::from_str(&raw).ok()?;
    let _ = fs::remove_file(path);
    Some(filter)
}

fn write_logr_filter_request(filter: &super::actions::LogFilter) -> Result<(), String> {
    let path = logr_filter_request_path();
    super::ensure_parent(&path);
    let payload = serde_json::to_string(filter).map_err(|error| error.to_string())?;
    fs::write(path, payload).map_err(|error| error.to_string())
}

fn write_annotation(hunk: &HunkRef, note: &str) -> Result<(), String> {
    let context = build_workspace_context();
    let repo_root = context
        .git
        .repo_root
        .ok_or_else(|| "no active git repository".to_string())?;
    let path = patch_session_path(Path::new(&repo_root), &hunk.branch);
    let raw = fs::read_to_string(&path).map_err(|error| format!("patch session read failed: {error}"))?;
    let mut json: Value = serde_json::from_str(&raw).map_err(|error| format!("patch session parse failed: {error}"))?;
    let hunks = json
        .get_mut("hunks")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "patch session missing hunks".to_string())?;
    let target = hunks.iter_mut().find(|item| {
        item.get("file").and_then(Value::as_str) == Some(hunk.file.as_str())
            && item.get("hunk_index").and_then(Value::as_u64) == Some(hunk.hunk_index as u64)
    });
    let Some(target) = target else {
        return Err(format!("patch hunk not found: {}#{}", hunk.file, hunk.hunk_index));
    };
    if let Some(object) = target.as_object_mut() {
        object.insert("annotation".to_string(), Value::String(note.to_string()));
    }
    fs::write(path, serde_json::to_string_pretty(&json).map_err(|error| error.to_string())?)
        .map_err(|error| format!("patch session write failed: {error}"))
}

fn stage_hunk(hunk: &HunkRef) -> Result<(), String> {
    let context = build_workspace_context();
    let repo_root = context
        .git
        .repo_root
        .ok_or_else(|| "no active git repository".to_string())?;
    let patch = select_hunk_patch(Path::new(&repo_root), &hunk.file, hunk.hunk_index)?;
    let mut child = Command::new("git")
        .args(["apply", "--cached", "--recount", "-"])
        .current_dir(&repo_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start git apply: {error}"))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(patch.as_bytes())
            .map_err(|error| format!("failed to write patch to git apply: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("git apply failed to finish: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn select_hunk_patch(repo_root: &Path, file: &str, hunk_index: usize) -> Result<String, String> {
    let raw = Command::new("git")
        .args(["diff", "--color=never", "-U3", "--", file])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("failed to read git diff: {error}"))?;
    if !raw.status.success() {
        return Err(String::from_utf8_lossy(&raw.stderr).trim().to_string());
    }
    let diff = String::from_utf8_lossy(&raw.stdout).to_string();
    let mut header = Vec::new();
    let mut hunks = Vec::new();
    let mut current = Vec::new();
    let mut in_hunk = false;
    for line in diff.lines() {
        if line.starts_with("@@") {
            if in_hunk && !current.is_empty() {
                hunks.push(current.join("\n"));
                current.clear();
            }
            in_hunk = true;
            current.push(line.to_string());
            continue;
        }
        if !in_hunk {
            header.push(line.to_string());
            continue;
        }
        if line.starts_with("diff --git ") && !current.is_empty() {
            hunks.push(current.join("\n"));
            current.clear();
            header = vec![line.to_string()];
            in_hunk = false;
            continue;
        }
        current.push(line.to_string());
    }
    if !current.is_empty() {
        hunks.push(current.join("\n"));
    }
    let selected = hunks
        .get(hunk_index)
        .ok_or_else(|| format!("hunk {hunk_index} not found in {file}"))?;
    Ok(format!("{}\n{}\n", header.join("\n"), selected))
}
