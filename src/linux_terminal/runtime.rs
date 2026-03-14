use std::{
    env,
    path::{Path, PathBuf},
};

const BUNDLED_SHELL_ENV: &str = "OBSIDIAN_BUNDLED_SHELL";

pub(super) fn resolve_shell(shell_override: &str) -> String {
    if let Some(path) = resolve_executable(shell_override) {
        return path.display().to_string();
    }

    if let Some(path) = bundled_shell() {
        return path.display().to_string();
    }

    if let Ok(shell) = env::var("SHELL") {
        if let Some(path) = resolve_executable(&shell) {
            return path.display().to_string();
        }
    }

    for fallback in ["/bin/bash", "/bin/sh"] {
        if let Some(path) = resolve_executable(fallback) {
            return path.display().to_string();
        }
    }

    shell_override.to_string()
}

pub(super) fn bundled_shell() -> Option<PathBuf> {
    if let Ok(path) = env::var(BUNDLED_SHELL_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    bundled_shell_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

fn bundled_shell_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(appdir) = env::var_os("APPDIR").map(PathBuf::from) {
        candidates.push(appdir.join("usr/lib/obsidian/bin/bash"));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("bash"));
            candidates.push(dir.join("../lib/obsidian/bin/bash"));
            candidates.push(dir.join("../Resources/bin/bash"));
        }
    }

    candidates
}

pub(super) fn resolve_executable(input: &str) -> Option<PathBuf> {
    if input.trim().is_empty() {
        return None;
    }

    let path = PathBuf::from(input);
    if path.components().count() > 1 || input.starts_with('.') || input.starts_with('/') {
        return path.is_file().then_some(path);
    }

    let path_env = env::var_os("PATH")?;
    env::split_paths(&path_env)
        .map(|dir| dir.join(input))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}
