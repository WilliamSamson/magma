use std::fs;

use serde::{Deserialize, Serialize};

use crate::linux_terminal::settings::{settings_path, Settings};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SetupCheckpoint {
    settings: Settings,
    step: u32,
}

pub(super) fn load(initial_settings: &Settings) -> (Settings, u32) {
    let path = checkpoint_path();
    match fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str::<SetupCheckpoint>(&contents) {
            Ok(checkpoint) => (checkpoint.settings, checkpoint.step),
            Err(_) => (initial_settings.clone(), 0),
        },
        Err(_) => (initial_settings.clone(), 0),
    }
}

pub(super) fn save(settings: &Settings, step: u32) {
    let path = checkpoint_path();
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create checkpoint directory: {error}");
            return;
        }
    }

    let checkpoint = SetupCheckpoint {
        settings: settings.clone(),
        step,
    };
    match serde_json::to_string_pretty(&checkpoint) {
        Ok(json) => {
            if let Err(error) = fs::write(&path, json) {
                eprintln!("failed to save setup checkpoint: {error}");
            }
        }
        Err(error) => eprintln!("failed to serialize setup checkpoint: {error}"),
    }
}

pub(super) fn clear() {
    let path = checkpoint_path();
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            eprintln!("setup checkpoint cleanup failed: {error}");
        }
    }
}

fn checkpoint_path() -> std::path::PathBuf {
    settings_path()
        .parent()
        .map(|parent| parent.join("setup-checkpoint.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("setup-checkpoint.json"))
}
