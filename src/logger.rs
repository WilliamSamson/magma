use std::{
    env,
    fs,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::SystemTime,
};

use serde_json::json;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

const LOG_FILENAME: &str = "magma-debug.log.jsonl";

pub(crate) fn init() {
    let Some((path, file)) = open_log_file() else {
        eprintln!("logger: could not open a log file");
        return;
    };

    {
        let Ok(mut guard) = LOG_FILE.lock() else {
            eprintln!("logger: log file lock is poisoned");
            return;
        };
        *guard = Some(file);
    }
    info("magma started", &[("log_path", path.to_string_lossy().as_ref())]);
}

pub(crate) fn info(message: &str, fields: &[(&str, &str)]) {
    write_entry("info", message, fields);
}

#[allow(dead_code)]
pub(crate) fn warn(message: &str, fields: &[(&str, &str)]) {
    write_entry("warn", message, fields);
}

pub(crate) fn error(message: &str, fields: &[(&str, &str)]) {
    write_entry("error", message, fields);
}

#[allow(dead_code)]
pub(crate) fn debug(message: &str, fields: &[(&str, &str)]) {
    write_entry("debug", message, fields);
}

fn open_log_file() -> Option<(PathBuf, std::fs::File)> {
    let mut last_error = None;

    for path in candidate_log_paths() {
        if let Some(parent) = path.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            last_error = Some(format!("could not create {}: {error}", parent.display()));
            continue;
        }

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(file) => return Some((path, file)),
            Err(error) => {
                last_error = Some(format!("could not open {}: {error}", path.display()));
            }
        }
    }

    if let Some(error) = last_error {
        eprintln!("logger: {error}");
    }

    None
}

fn candidate_log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(state_home) = env::var_os("XDG_STATE_HOME") {
        paths.push(PathBuf::from(state_home).join("magma").join(LOG_FILENAME));
    } else if let Some(home) = env::var_os("HOME") {
        paths.push(
            PathBuf::from(home)
                .join(".local")
                .join("state")
                .join("magma")
                .join(LOG_FILENAME),
        );
    }

    paths.push(env::temp_dir().join(LOG_FILENAME));
    paths
}

fn write_entry(level: &str, message: &str, fields: &[(&str, &str)]) {
    let Ok(mut guard) = LOG_FILE.lock() else {
        return;
    };
    let Some(file) = guard.as_mut() else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut entry = serde_json::Map::new();
    entry.insert("timestamp".into(), json!(timestamp));
    entry.insert("level".into(), json!(level));
    entry.insert("message".into(), json!(message));
    entry.insert("component".into(), json!("magma"));
    for (key, value) in fields {
        entry.insert((*key).to_string(), json!(value));
    }

    if let Ok(line) = serde_json::to_string(&entry) {
        let _ = writeln!(file, "{line}");
    }
}
