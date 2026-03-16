pub(crate) mod actions;
pub(crate) mod context;
pub(crate) mod effects;
pub(crate) mod executor;
pub(crate) mod memory;
pub(crate) mod model;
pub(crate) mod observer;

use std::{
    env,
    path::{Path, PathBuf},
};

pub(crate) fn data_root() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("magma")
}

pub(crate) fn ensure_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

pub(crate) fn patch_session_path(repo_root: &Path, branch: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    repo_root.hash(&mut hasher);
    data_root()
        .join("patch")
        .join(format!("{:016x}", hasher.finish()))
        .join(format!("{}.json", branch.replace('/', "_")))
}
