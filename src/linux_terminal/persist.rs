use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::profile::ProfileId;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub(super) struct WorkspaceSnapshot {
    pub(super) active_tab: usize,
    pub(super) tabs: Vec<TabSnapshot>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum PaneFocus {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct TabSnapshot {
    pub(super) title: String,
    pub(super) profile: ProfileId,
    pub(super) left_cwd: Option<String>,
    pub(super) right_cwd: Option<String>,
    pub(super) split_position: Option<i32>,
    pub(super) active_pane: PaneFocus,
}

impl Default for TabSnapshot {
    fn default() -> Self {
        Self {
            title: "tab 1".to_string(),
            profile: ProfileId::Default,
            left_cwd: None,
            right_cwd: None,
            split_position: None,
            active_pane: PaneFocus::Left,
        }
    }
}

pub(super) fn load_workspace() -> io::Result<Option<WorkspaceSnapshot>> {
    let path = workspace_file();
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)?;
    let snapshot =
        serde_json::from_str::<WorkspaceSnapshot>(&contents).map_err(io::Error::other)?;
    Ok(Some(snapshot))
}

pub(super) fn save_workspace(snapshot: &WorkspaceSnapshot) -> io::Result<()> {
    let path = workspace_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, json)
}

fn workspace_file() -> PathBuf {
    config_root().join("workspace.json")
}

fn config_root() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    base.join("obsidian")
}
