mod about;
mod browser;
mod page;
mod search;
mod sections;
mod terminal;
mod widgets;

use std::{
    env,
    fs,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::theme::ThemeMode;

pub(super) use page::build_settings_page;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) font_family: String,
    pub(crate) font_size: u32,
    pub(crate) app_font_size: u32,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) default_browser: String,
    pub(crate) scrollback_lines: u32,
    pub(crate) cursor_style: String,
    pub(crate) cursor_blink: bool,
    pub(crate) image_rendering: bool,
    pub(crate) ligatures: bool,
    pub(crate) shell: String,
    pub(crate) logr_panel_open: bool,
    pub(crate) notifications: bool,
    pub(crate) agent_confidence_threshold: f32,
    pub(crate) agent_passive_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        Self {
            font_family: "DejaVu Sans Mono".to_string(),
            font_size: 9,
            app_font_size: 10,
            theme_mode: ThemeMode::Dark,
            default_browser: "google".to_string(),
            scrollback_lines: 20_000,
            cursor_style: "ibeam".to_string(),
            cursor_blink: false,
            image_rendering: true,
            ligatures: true,
            shell,
            logr_panel_open: true,
            notifications: true,
            agent_confidence_threshold: 0.85,
            agent_passive_mode: false,
        }
    }
}

pub(crate) fn load_settings() -> Settings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub(crate) fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create settings directory: {error}");
            return;
        }
    }
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(error) = fs::write(&path, json) {
                eprintln!("failed to save settings to {}: {error}", path.display());
            }
        }
        Err(error) => eprintln!("failed to serialize settings: {error}"),
    }
}

pub(crate) fn settings_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("magma").join("settings.json")
}

pub(crate) fn settings_exist() -> bool {
    settings_path().is_file()
}
