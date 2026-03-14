use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Deserializer, Serialize};

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
    #[serde(alias = "left_cwd", deserialize_with = "deserialize_optional_pane")]
    pub(super) left_pane: Option<PaneSnapshot>,
    #[serde(alias = "right_cwd", deserialize_with = "deserialize_optional_pane")]
    pub(super) right_pane: Option<PaneSnapshot>,
    pub(super) split_position: Option<i32>,
    pub(super) active_pane: PaneFocus,
}

impl Default for TabSnapshot {
    fn default() -> Self {
        Self {
            title: "tab 1".to_string(),
            profile: ProfileId::Default,
            left_pane: Some(PaneSnapshot::default()),
            right_pane: None,
            split_position: None,
            active_pane: PaneFocus::Left,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct PaneSnapshot {
    pub(super) sessions: Vec<SessionSnapshot>,
    pub(super) active_session: usize,
}

impl Default for PaneSnapshot {
    fn default() -> Self {
        Self::from_cwd(None)
    }
}

impl PaneSnapshot {
    pub(super) fn from_cwd(cwd: Option<String>) -> Self {
        Self {
            sessions: vec![SessionSnapshot::new(cwd)],
            active_session: 0,
        }
    }

    pub(super) fn normalized(mut self) -> Self {
        if self.sessions.is_empty() {
            self.sessions.push(SessionSnapshot::default());
        }
        self.sessions = self
            .sessions
            .into_iter()
            .map(SessionSnapshot::normalized)
            .collect();
        self.active_session = self
            .active_session
            .min(self.sessions.len().saturating_sub(1));
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct SessionSnapshot {
    pub(super) cwd: Option<String>,
    pub(super) session_id: Option<String>,
    pub(super) socket_path: Option<String>,
    pub(super) status_path: Option<String>,
}

impl SessionSnapshot {
    pub(super) fn new(cwd: Option<String>) -> Self {
        Self {
            cwd,
            session_id: Some(generate_session_id()),
            socket_path: Some(default_socket_path().display().to_string()),
            status_path: Some(default_status_path().display().to_string()),
        }
    }

    pub(super) fn normalized(mut self) -> Self {
        if self.session_id.is_none() {
            self.session_id = Some(generate_session_id());
        }

        if self.socket_path.is_none() {
            self.socket_path = Some(default_socket_path().display().to_string());
        }

        if self.status_path.is_none() {
            self.status_path = Some(default_status_path().display().to_string());
        }

        self
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

fn default_socket_path() -> PathBuf {
    if let Some(path) = env::var_os("OBSIDIAN_TMUX_SOCKET").map(PathBuf::from) {
        return path;
    }

    runtime_root().join("tmux.sock")
}

fn default_status_path() -> PathBuf {
    state_root().join(format!("shell_status_{}.tsv", timestamp_nanos()))
}

fn state_root() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(std::env::temp_dir)
        .join("obsidian")
}

fn runtime_root() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| state_root().join("run"))
        .join("obsidian")
}

fn generate_session_id() -> String {
    format!(
        "obsidian-{}-{}",
        std::process::id(),
        timestamp_nanos()
    )
}

fn timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PaneSnapshotField {
    Pane(PaneSnapshot),
    LegacyCwd(String),
}

fn deserialize_optional_pane<'de, D>(deserializer: D) -> Result<Option<PaneSnapshot>, D::Error>
where
    D: Deserializer<'de>,
{
    let field = Option::<PaneSnapshotField>::deserialize(deserializer)?;
    Ok(field.map(|value| match value {
        PaneSnapshotField::Pane(pane) => pane.normalized(),
        PaneSnapshotField::LegacyCwd(cwd) => PaneSnapshot::from_cwd(Some(cwd)),
    }))
}

fn config_root() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    base.join("obsidian")
}
