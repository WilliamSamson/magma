use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use super::{data_root, ensure_parent};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SessionMemory {
    pub(crate) branch: String,
    pub(crate) summary: String,
    pub(crate) timestamp_ms: u128,
}

pub(crate) fn repo_memory_dir(repo_root: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_root.hash(&mut hasher);
    data_root()
        .join("memory")
        .join(format!("{:016x}", hasher.finish()))
}

pub(crate) fn load_recent_summaries(repo_root: &Path, branch: &str, count: usize) -> Vec<String> {
    let path = repo_memory_dir(repo_root).join(format!("{}.json", sanitize(branch)));
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(mut items) = serde_json::from_str::<Vec<SessionMemory>>(&raw) else {
        return Vec::new();
    };
    items.sort_by_key(|item| item.timestamp_ms);
    items.into_iter().rev().take(count).map(|item| item.summary).collect()
}

#[allow(dead_code)]
pub(crate) fn store_summary(repo_root: &Path, branch: &str, summary: String) -> std::io::Result<()> {
    let path = repo_memory_dir(repo_root).join(format!("{}.json", sanitize(branch)));
    ensure_parent(&path);
    let mut entries = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<SessionMemory>>(&raw).ok())
        .unwrap_or_default();
    entries.push(SessionMemory {
        branch: branch.to_string(),
        summary,
        timestamp_ms: now_ms(),
    });
    if entries.len() > 24 {
        let drain = entries.len() - 24;
        entries.drain(0..drain);
    }
    let json = serde_json::to_string_pretty(&entries).map_err(std::io::Error::other)?;
    fs::write(path, json)
}

fn sanitize(branch: &str) -> String {
    branch.replace('/', "_")
}

#[allow(dead_code)]
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
