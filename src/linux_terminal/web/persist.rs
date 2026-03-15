use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct WebSnapshot {
    pub(super) active_tab: usize,
    pub(super) tabs: Vec<WebTabSnapshot>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct WebTabSnapshot {
    pub(super) uri: String,
}

pub(super) fn load_snapshot() -> io::Result<Option<WebSnapshot>> {
    let path = snapshot_path();
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)?;
    let snapshot = serde_json::from_str::<WebSnapshot>(&contents).map_err(io::Error::other)?;
    Ok(Some(snapshot))
}

pub(super) fn save_snapshot(snapshot: &WebSnapshot) -> io::Result<()> {
    let path = snapshot_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(snapshot).map_err(io::Error::other)?;
    fs::write(path, json)
}

pub(super) fn clear_snapshot() -> io::Result<()> {
    let path = snapshot_path();
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
}

fn snapshot_path() -> PathBuf {
    config_root().join("web.json")
}

fn config_root() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    base.join("magma")
}
