use std::{
    env,
    fs,
    io,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;

#[derive(Serialize, Deserialize)]
struct StoredWindowState {
    width: u32,
    height: u32,
}

pub(crate) fn load_window_size() -> io::Result<Option<PhysicalSize<u32>>> {
    let Some(path) = state_path() else {
        return Ok(None);
    };
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let state: StoredWindowState =
        serde_json::from_str(&contents).map_err(io::Error::other)?;
    if state.width == 0 || state.height == 0 {
        return Ok(None);
    }
    Ok(Some(PhysicalSize::new(state.width, state.height)))
}

pub(crate) fn save_window_size(size: PhysicalSize<u32>) -> io::Result<()> {
    if size.width == 0 || size.height == 0 {
        return Ok(());
    }

    let Some(path) = state_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let state = StoredWindowState {
        width: size.width,
        height: size.height,
    };
    let json = serde_json::to_string(&state).map_err(io::Error::other)?;
    fs::write(path, json)
}

fn state_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(config_home);
        path.push("magma");
        path.push("window-state.json");
        return Some(path);
    }

    let home = env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("magma");
    path.push("window-state.json");
    Some(path)
}
