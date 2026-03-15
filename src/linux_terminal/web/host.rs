use std::{
    cell::{Cell, RefCell},
    env, fs,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::OpenOptions,
};

use gtk::{prelude::*, Box as GtkBox, Orientation, Overflow};
use webkit6::{CookiePersistentStorage, NetworkSession, WebContext};

use crate::linux_terminal::settings::Settings;

use super::{build_web_pane, persist};

#[derive(Clone)]
pub(in crate::linux_terminal) struct WebPaneHost {
    root: GtkBox,
    // Rc<Cell<bool>> is enough here because lazy-load state is a small main-thread boolean shared across callbacks.
    loaded: Rc<Cell<bool>>,
    settings: Rc<RefCell<Settings>>,
    context: Rc<RefCell<WebContext>>,
    network_session: Rc<RefCell<NetworkSession>>,
}

impl WebPaneHost {
    pub(in crate::linux_terminal) fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_width_request(0);
        root.set_overflow(Overflow::Hidden);

        Self {
            root,
            loaded: Rc::new(Cell::new(false)),
            settings,
            context: Rc::new(RefCell::new(WebContext::new())),
            network_session: Rc::new(RefCell::new(persistent_network_session())),
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        // WebContext clone keeps one shared WebKit context handle across lazy-created tabs in one browser pane.
        let pane = build_web_pane(
            self.settings.clone(),
            self.context.borrow().clone(),
            self.network_session.borrow().clone(),
        );
        self.root.append(&pane.root);
    }

    pub(in crate::linux_terminal) fn clear_persistent_data(&self) {
        self.reset_browser_surface();
        clear_browser_storage();
        *self.context.borrow_mut() = WebContext::new();
        *self.network_session.borrow_mut() = persistent_network_session();

        if self.loaded.get() {
            let pane = build_web_pane(
                self.settings.clone(),
                self.context.borrow().clone(),
                self.network_session.borrow().clone(),
            );
            self.root.append(&pane.root);
        }
    }

    fn reset_browser_surface(&self) {
        while let Some(child) = self.root.first_child() {
            self.root.remove(&child);
        }
    }
}

fn persistent_network_session() -> NetworkSession {
    let data_dir = web_data_dir();
    let cache_dir = web_cache_dir();
    ensure_private_dir(&data_dir);
    ensure_private_dir(&cache_dir);

    let data_dir_text = data_dir.to_string_lossy().into_owned();
    let cache_dir_text = cache_dir.to_string_lossy().into_owned();
    let session = NetworkSession::new(Some(&data_dir_text), Some(&cache_dir_text));
    session.set_persistent_credential_storage_enabled(true);

    if let Some(cookie_manager) = session.cookie_manager() {
        let cookie_path = data_dir.join("cookies.sqlite");
        ensure_private_file(&cookie_path);
        let cookie_path_text = cookie_path.to_string_lossy().into_owned();
        cookie_manager.set_persistent_storage(&cookie_path_text, CookiePersistentStorage::Sqlite);
    }

    session
}

fn web_data_dir() -> PathBuf {
    data_root().join(".web-profile")
}

fn web_cache_dir() -> PathBuf {
    cache_root().join(".web-profile")
}

fn data_root() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("magma")
}

fn cache_root() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join("magma")
}

fn clear_browser_storage() {
    if let Err(error) = persist::clear_snapshot() {
        eprintln!("web snapshot clear failed: {error}");
    }

    for path in [web_data_dir(), web_cache_dir()] {
        if !path.exists() {
            continue;
        }

        if let Err(error) = fs::remove_dir_all(&path) {
            eprintln!("web profile clear failed for {}: {error}", path.display());
        }
    }
}

fn ensure_private_dir(path: &Path) {
    if let Err(error) = fs::create_dir_all(path) {
        eprintln!("web profile directory create failed for {}: {error}", path.display());
        return;
    }

    #[cfg(unix)]
    if let Err(error) = fs::set_permissions(path, fs::Permissions::from_mode(0o700)) {
        eprintln!(
            "web profile directory permission update failed for {}: {error}",
            path.display()
        );
    }
}

fn ensure_private_file(path: &Path) {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent);
    }

    if let Err(error) = create_private_file(path) {
        eprintln!("web profile file prepare failed for {}: {error}", path.display());
    }
}

fn create_private_file(path: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);

    #[cfg(unix)]
    options.mode(0o600);

    options.open(path)?;

    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;

    Ok(())
}
