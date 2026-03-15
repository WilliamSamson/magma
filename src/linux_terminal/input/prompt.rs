use std::path::{Path, PathBuf};

use gtk::{gio, pango, prelude::*, Box as GtkBox, Label, Orientation};
use vte4::{prelude::*, Terminal};

pub(super) fn build_prompt_box(terminal: &Terminal) -> GtkBox {
    let prompt_container = GtkBox::new(Orientation::Horizontal, 6);

    let user_label = Label::new(Some(&current_username()));
    user_label.add_css_class("magma-user-label");
    prompt_container.append(&user_label);

    let path_label = Label::new(Some(&current_path_display(terminal)));
    path_label.add_css_class("magma-path-label");
    path_label.set_ellipsize(pango::EllipsizeMode::Middle);
    path_label.set_xalign(0.0);
    prompt_container.append(&path_label);

    let prompt_label = Label::new(Some(">"));
    prompt_label.add_css_class("magma-user-label");
    prompt_container.append(&prompt_label);

    connect_directory_updates(terminal, &path_label);
    prompt_container
}

fn connect_directory_updates(terminal: &Terminal, path_label: &Label) {
    let path_label = path_label.clone();
    terminal.connect_current_directory_uri_changed(move |terminal| {
        path_label.set_text(&current_path_display(terminal));
    });
}

fn current_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "user".to_string())
}

fn current_path_display(terminal: &Terminal) -> String {
    terminal
        .current_directory_uri()
        .as_deref()
        .and_then(path_from_uri)
        .unwrap_or_else(fallback_path)
}

fn path_from_uri(uri: &str) -> Option<String> {
    let path = gio::File::for_uri(uri).path()?;
    Some(compact_path(&path))
}

fn fallback_path() -> String {
    std::env::current_dir()
        .ok()
        .map(|path| compact_path(&path))
        .unwrap_or_else(|| "~".to_string())
}

fn compact_path(path: &Path) -> String {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return path.display().to_string();
    };

    match path.strip_prefix(&home) {
        Ok(stripped) if stripped.as_os_str().is_empty() => "~".to_string(),
        Ok(stripped) => format!("~/{}", stripped.display()),
        Err(_) => path.display().to_string(),
    }
}
