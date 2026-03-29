use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    time::UNIX_EPOCH,
};

use gtk::{
    glib,
    prelude::*, Align, Box as GtkBox, Image, Label, Orientation, ScrolledWindow,
    PolicyType,
};

use super::view::CwdProvider;

const INDENT_PX: i32 = 16;
const CWD_POLL_MS: u64 = 800;

/// Callback invoked when a file is clicked in the explorer.
pub(super) type OnFileClick = Rc<dyn Fn(&Path)>;

/// Builds the folder tree pane widget.
pub(super) fn build_folder_pane(cwd_provider: CwdProvider, on_file_click: OnFileClick) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    let header = build_header();
    root.append(&header);

    // Breadcrumb bar
    let breadcrumb = GtkBox::new(Orientation::Horizontal, 2);
    breadcrumb.add_css_class("magma-breadcrumb");
    breadcrumb.set_hexpand(true);

    let breadcrumb_scroll = ScrolledWindow::new();
    breadcrumb_scroll.set_hexpand(true);
    breadcrumb_scroll.set_policy(PolicyType::Automatic, PolicyType::Never);
    breadcrumb_scroll.set_child(Some(&breadcrumb));
    root.append(&breadcrumb_scroll);

    let tree_box = GtkBox::new(Orientation::Vertical, 0);
    tree_box.set_vexpand(true);
    tree_box.add_css_class("magma-folder-tree");

    let scroll = ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_child(Some(&tree_box));
    root.append(&scroll);

    let last_cwd: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // Manual refresh
    let refresh_btn = header
        .first_child()
        .and_then(|w| w.next_sibling())
        .and_then(|w| w.last_child());

    if let Some(btn) = refresh_btn {
        if let Ok(button) = btn.downcast::<gtk::Button>() {
            let cwd_provider = cwd_provider.clone();
            let tree_box_ref = tree_box.clone();
            let breadcrumb_ref = breadcrumb.clone();
            let last_cwd = last_cwd.clone();
            let on_file_click = on_file_click.clone();
            button.connect_clicked(move |_| {
                *last_cwd.borrow_mut() = None;
                populate_tree(&tree_box_ref, &breadcrumb_ref, &cwd_provider, &on_file_click);
                *last_cwd.borrow_mut() = cwd_provider();
            });
        }
    }

    // Auto-sync
    {
        let tree_box_ref = tree_box.clone();
        let breadcrumb_ref = breadcrumb.clone();
        let cwd_provider = cwd_provider.clone();
        let last_cwd = last_cwd.clone();
        let on_file_click = on_file_click.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(CWD_POLL_MS), move || {
            let current = cwd_provider();
            let prev = last_cwd.borrow().clone();
            if current != prev {
                populate_tree(&tree_box_ref, &breadcrumb_ref, &cwd_provider, &on_file_click);
                *last_cwd.borrow_mut() = current;
            }
            glib::ControlFlow::Continue
        });
    }

    root
}

fn build_header() -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.add_css_class("magma-folder-header");
    header.set_margin_bottom(4);

    let title = Label::new(Some("Explorer"));
    title.add_css_class("magma-folder-title");
    title.set_halign(Align::Start);
    title.set_hexpand(true);

    let actions = GtkBox::new(Orientation::Horizontal, 4);
    actions.set_halign(Align::End);

    let refresh = gtk::Button::builder()
        .css_classes(["magma-folder-action"])
        .tooltip_text("Refresh")
        .build();
    let refresh_icon = Image::from_icon_name("view-refresh-symbolic");
    refresh_icon.add_css_class("magma-folder-action-icon");
    refresh.set_child(Some(&refresh_icon));
    actions.append(&refresh);

    header.append(&title);
    header.append(&actions);
    header
}

// ── Breadcrumb ──

fn build_breadcrumbs(container: &GtkBox, cwd: &Path) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let display_path = cwd.display().to_string();
    let shortened = if !home.is_empty() && display_path.starts_with(&home) {
        format!("~{}", &display_path[home.len()..])
    } else {
        display_path
    };

    let segments: Vec<&str> = shortened.split('/').filter(|s| !s.is_empty()).collect();

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            let sep = Label::new(Some("/"));
            sep.add_css_class("magma-breadcrumb-sep");
            container.append(&sep);
        }

        let label = Label::new(Some(segment));
        label.add_css_class("magma-breadcrumb-segment");
        if i == segments.len() - 1 {
            label.add_css_class("magma-breadcrumb-active");
        }
        label.set_ellipsize(gtk::pango::EllipsizeMode::None);
        container.append(&label);
    }
}

// ── Git status ──

#[derive(Clone, Copy, PartialEq, Eq)]
enum GitStatus {
    Modified,
    Staged,
    Untracked,
    Conflict,
}

fn collect_git_status(cwd: &Path) -> HashMap<PathBuf, GitStatus> {
    let mut map = HashMap::new();

    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["status", "--porcelain", "-uall"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return map,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find git root so we can produce absolute paths
    let git_root = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
            } else {
                None
            }
        });

    let root = match git_root {
        Some(r) => r,
        None => return map,
    };

    for line in stdout.lines() {
        if line.len() < 4 {
            continue;
        }

        let index = line.as_bytes()[0];
        let worktree = line.as_bytes()[1];
        let file_path = line[3..].trim();

        let status = match (index, worktree) {
            (b'U', _) | (_, b'U') | (b'A', b'A') | (b'D', b'D') => GitStatus::Conflict,
            (b'A', _) | (b'M', b' ') | (b'D', b' ') | (b'R', _) | (b'C', _) => GitStatus::Staged,
            (_, b'M') | (_, b'D') => GitStatus::Modified,
            (b'?', b'?') => GitStatus::Untracked,
            _ => continue,
        };

        let abs_path = root.join(file_path);
        map.insert(abs_path, status);
    }

    map
}

fn git_status_for_dir(git_map: &HashMap<PathBuf, GitStatus>, dir: &Path) -> Option<GitStatus> {
    let mut worst: Option<GitStatus> = None;
    for (path, status) in git_map {
        if path.starts_with(dir) {
            let priority = |s: GitStatus| match s {
                GitStatus::Conflict => 3,
                GitStatus::Modified => 2,
                GitStatus::Staged => 1,
                GitStatus::Untracked => 0,
            };
            worst = Some(match worst {
                None => *status,
                Some(w) if priority(*status) > priority(w) => *status,
                Some(w) => w,
            });
        }
    }
    worst
}

fn git_status_css(status: GitStatus) -> &'static str {
    match status {
        GitStatus::Modified => "git-modified",
        GitStatus::Staged => "git-staged",
        GitStatus::Untracked => "git-untracked",
        GitStatus::Conflict => "git-conflict",
    }
}

// ── Tree building ──

fn populate_tree(
    container: &GtkBox,
    breadcrumb: &GtkBox,
    cwd_provider: &CwdProvider,
    on_file_click: &OnFileClick,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let cwd = match cwd_provider() {
        Some(path) => PathBuf::from(path),
        None => {
            let label = Label::new(Some("No working directory"));
            label.add_css_class("magma-folder-empty");
            container.append(&label);
            build_breadcrumbs(breadcrumb, Path::new(""));
            return;
        }
    };

    if !cwd.is_dir() {
        let label = Label::new(Some("Not a directory"));
        label.add_css_class("magma-folder-empty");
        container.append(&label);
        return;
    }

    build_breadcrumbs(breadcrumb, &cwd);

    let git_map = Rc::new(collect_git_status(&cwd));
    build_tree_level(container, &cwd, 0, &git_map, on_file_click);
}

fn build_tree_level(
    container: &GtkBox,
    dir: &Path,
    depth: u32,
    git_map: &Rc<HashMap<PathBuf, GitStatus>>,
    on_file_click: &OnFileClick,
) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };

    entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let entry_path = entry.path();

        if is_dir {
            let dir_git = git_status_for_dir(git_map, &entry_path);
            let folder_row = build_folder_row(&name, depth, dir_git);
            let children_box = GtkBox::new(Orientation::Vertical, 0);
            children_box.set_visible(false);

            let expanded = Rc::new(std::cell::Cell::new(false));
            let children_ref = children_box.clone();
            let git_map = git_map.clone();
            let on_file_click = on_file_click.clone();

            if let Some(btn) = folder_row.first_child() {
                if let Ok(button) = btn.downcast::<gtk::Button>() {
                    button.connect_clicked(move |btn| {
                        let is_expanded = expanded.get();
                        if is_expanded {
                            children_ref.set_visible(false);
                            expanded.set(false);
                            update_chevron(btn, "pan-end-symbolic");
                        } else {
                            if children_ref.first_child().is_none() {
                                build_tree_level(
                                    &children_ref,
                                    &entry_path,
                                    depth + 1,
                                    &git_map,
                                    &on_file_click,
                                );
                            }
                            children_ref.set_visible(true);
                            expanded.set(true);
                            update_chevron(btn, "pan-down-symbolic");
                        }
                    });
                }
            }

            container.append(&folder_row);
            container.append(&children_box);
        } else {
            let file_git = git_map.get(&entry_path).copied();
            let tooltip = file_tooltip(&entry_path);
            let file_row = build_file_row(&name, depth, file_git, &tooltip);

            // Wire click-to-open
            let path = entry_path.clone();
            let on_file_click = on_file_click.clone();
            if let Some(btn) = file_row.first_child() {
                if let Ok(button) = btn.downcast::<gtk::Button>() {
                    button.connect_clicked(move |_| {
                        on_file_click(&path);
                    });
                }
            }

            container.append(&file_row);
        }
    }
}

fn update_chevron(btn: &gtk::Button, icon_name: &str) {
    if let Some(row) = btn.child() {
        if let Ok(row_box) = row.downcast::<GtkBox>() {
            if let Some(chevron) = row_box.first_child() {
                if let Ok(img) = chevron.downcast::<Image>() {
                    img.set_icon_name(Some(icon_name));
                }
            }
        }
    }
}

// ── File tooltips ──

fn file_tooltip(path: &Path) -> String {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };

    let size = format_file_size(meta.len());
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| format_timestamp(d.as_secs()))
        .unwrap_or_else(|| "unknown".to_string());

    format!("{size}  ·  {modified}")
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn format_timestamp(secs: u64) -> String {
    // Simple date formatting without chrono: YYYY-MM-DD HH:MM
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Days since Unix epoch to calendar date (simplified)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let mut m = 1u32;
    let month_days = [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}")
}

fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

// ── Row builders ──

fn build_folder_row(name: &str, depth: u32, git: Option<GitStatus>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.set_margin_start(depth as i32 * INDENT_PX);

    let button = gtk::Button::builder()
        .css_classes(["magma-folder-item", "magma-folder-dir"])
        .build();
    button.set_hexpand(true);

    let content = GtkBox::new(Orientation::Horizontal, 6);
    content.set_halign(Align::Start);
    content.set_hexpand(true);

    let chevron = Image::from_icon_name("pan-end-symbolic");
    chevron.add_css_class("magma-folder-chevron");

    let (icon_name, color_class) = folder_icon(name);
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("magma-folder-icon");
    icon.add_css_class(color_class);

    let label = Label::new(Some(name));
    label.add_css_class("magma-folder-name");
    label.add_css_class(color_class);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_hexpand(true);
    label.set_halign(Align::Start);

    content.append(&chevron);
    content.append(&icon);
    content.append(&label);

    if let Some(status) = git {
        let dot = Label::new(Some("●"));
        dot.add_css_class("magma-git-dot");
        dot.add_css_class(git_status_css(status));
        dot.set_halign(Align::End);
        content.append(&dot);
    }

    button.set_child(Some(&content));
    row.append(&button);
    row
}

fn build_file_row(name: &str, depth: u32, git: Option<GitStatus>, tooltip: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.set_margin_start(depth as i32 * INDENT_PX + 20);

    let button = gtk::Button::builder()
        .css_classes(["magma-folder-item", "magma-folder-file"])
        .build();
    button.set_hexpand(true);
    if !tooltip.is_empty() {
        button.set_tooltip_text(Some(tooltip));
    }

    let content = GtkBox::new(Orientation::Horizontal, 6);
    content.set_halign(Align::Start);
    content.set_hexpand(true);

    let (icon_name, color_class) = file_icon(name);
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("magma-folder-icon");
    icon.add_css_class(color_class);

    let label = Label::new(Some(name));
    label.add_css_class("magma-folder-name");
    label.add_css_class(color_class);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_hexpand(true);
    label.set_halign(Align::Start);

    content.append(&icon);
    content.append(&label);

    if let Some(status) = git {
        let dot = Label::new(Some("●"));
        dot.add_css_class("magma-git-dot");
        dot.add_css_class(git_status_css(status));
        dot.set_halign(Align::End);
        content.append(&dot);
    }

    button.set_child(Some(&content));
    row.append(&button);
    row
}

// ── File-type icon and color mapping ──

fn folder_icon(name: &str) -> (&'static str, &'static str) {
    match name.to_lowercase().as_str() {
        "src" | "lib" | "app" | "core" => ("folder-symbolic", "ft-source"),
        "test" | "tests" | "spec" | "specs" | "__tests__" => ("folder-symbolic", "ft-test"),
        "build" | "dist" | "out" | "target" | "bin" => ("folder-symbolic", "ft-build"),
        "node_modules" | "vendor" | ".cargo" | "deps" => ("folder-symbolic", "ft-dep"),
        "docs" | "doc" | "documentation" => ("folder-symbolic", "ft-doc"),
        "assets" | "static" | "public" | "images" | "img" | "media" => {
            ("folder-symbolic", "ft-asset")
        }
        ".git" | ".github" | ".gitlab" => ("folder-symbolic", "ft-git"),
        "config" | "configs" | ".config" => ("folder-symbolic", "ft-config"),
        _ => ("folder-symbolic", "ft-folder"),
    }
}

fn file_icon(name: &str) -> (&'static str, &'static str) {
    match name.to_lowercase().as_str() {
        "cargo.toml" | "cargo.lock" => return ("application-x-executable-symbolic", "ft-rust"),
        "package.json" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => {
            return ("application-x-executable-symbolic", "ft-js")
        }
        "tsconfig.json" => return ("application-x-executable-symbolic", "ft-ts"),
        "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => {
            return ("application-x-executable-symbolic", "ft-config")
        }
        ".gitignore" | ".gitmodules" | ".gitattributes" => {
            return ("emblem-shared-symbolic", "ft-git")
        }
        "makefile" | "cmakelists.txt" | "justfile" => {
            return ("application-x-executable-symbolic", "ft-build")
        }
        "readme.md" | "license" | "license.md" | "changelog.md" => {
            return ("accessories-text-editor-symbolic", "ft-doc")
        }
        ".env" | ".env.local" | ".env.example" => {
            return ("dialog-password-symbolic", "ft-secret")
        }
        _ => {}
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => ("text-x-generic-symbolic", "ft-rust"),
        "js" | "mjs" | "cjs" | "jsx" => ("text-x-generic-symbolic", "ft-js"),
        "ts" | "tsx" | "mts" | "cts" => ("text-x-generic-symbolic", "ft-ts"),
        "html" | "htm" | "svelte" | "vue" => ("text-html-symbolic", "ft-html"),
        "css" | "scss" | "sass" | "less" => ("text-x-generic-symbolic", "ft-css"),
        "json" | "jsonc" | "json5" => ("text-x-generic-symbolic", "ft-json"),
        "yaml" | "yml" => ("text-x-generic-symbolic", "ft-config"),
        "toml" | "ini" | "cfg" => ("text-x-generic-symbolic", "ft-config"),
        "xml" | "svg" => ("text-x-generic-symbolic", "ft-html"),
        "md" | "mdx" | "txt" | "rst" | "adoc" => ("accessories-text-editor-symbolic", "ft-doc"),
        "py" | "pyi" | "pyw" => ("text-x-generic-symbolic", "ft-python"),
        "go" | "mod" | "sum" => ("text-x-generic-symbolic", "ft-go"),
        "c" | "h" => ("text-x-generic-symbolic", "ft-c"),
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" => ("text-x-generic-symbolic", "ft-c"),
        "java" | "kt" | "kts" => ("text-x-generic-symbolic", "ft-java"),
        "sh" | "bash" | "zsh" | "fish" => ("utilities-terminal-symbolic", "ft-shell"),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" | "tiff" => {
            ("image-x-generic-symbolic", "ft-image")
        }
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => {
            ("package-x-generic-symbolic", "ft-archive")
        }
        "wasm" | "so" | "dylib" | "dll" | "exe" | "o" | "a" => {
            ("application-x-executable-symbolic", "ft-binary")
        }
        "lock" => ("dialog-password-symbolic", "ft-lock"),
        "sql" => ("text-x-generic-symbolic", "ft-config"),
        _ => ("text-x-generic-symbolic", "ft-default"),
    }
}
