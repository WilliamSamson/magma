mod code;
mod docx;
mod editor;
mod files;
mod host;
mod preview;
mod ui;

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use files::{category_label, category_order, scan_directory, FileCategory, ViewerFile};
use gtk::{
    glib, prelude::*, Box as GtkBox, Button, Label, ListBox, Orientation, Overflow, PolicyType,
    ScrolledWindow, SelectionMode,
};
use preview::{bind_open_external, build_preview, select_file, set_empty_preview, PreviewWidgets};
use ui::{build_header, clear_list, file_row, icon_button};
use webkit6::WebContext;

pub(super) use host::ViewPaneHost;
use host::OpenFileSlot;

pub(super) type CwdProvider = Rc<dyn Fn() -> Option<String>>;

fn build_view_pane(cwd_provider: CwdProvider, context: WebContext, open_file_slot: OpenFileSlot) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 8);
    root.add_css_class("magma-view-root");
    root.set_vexpand(true);
    root.set_overflow(Overflow::Hidden);

    let count_label = Label::new(Some(&format_file_count(0)));
    count_label.add_css_class("magma-view-count");

    let refresh_button = icon_button("view-refresh-symbolic", "Refresh files");
    let header = build_header(&refresh_button, &count_label);

    let scope_bar = GtkBox::new(Orientation::Horizontal, 4);
    scope_bar.add_css_class("magma-view-scope");

    let list = ListBox::new();
    list.add_css_class("magma-view-file-list");
    list.set_selection_mode(SelectionMode::Single);
    let list_scroller = ScrolledWindow::new();
    list_scroller.add_css_class("magma-view-file-scroller");
    list_scroller.set_min_content_height(140);
    list_scroller.set_max_content_height(180);
    list_scroller.set_policy(PolicyType::Never, PolicyType::Automatic);
    list_scroller.set_child(Some(&list));

    let preview = build_preview(context);
    root.append(&header);
    root.append(&scope_bar);
    root.append(&list_scroller);
    root.append(&preview.root);

    let state = Rc::new(ViewState {
        cwd_provider,
        model: RefCell::new(ViewModel::default()),
        widgets: ViewWidgets {
            list: list.clone(),
            count_label,
            preview,
            scope_bar,
        },
    });

    bind_selection(&state);
    editor::bind_editor_actions(&state);
    bind_open_external(&state);
    bind_refresh(&state, &refresh_button);
    refresh_from_terminal(&state, true);
    watch_terminal_directory(&state);
    watch_open_file_slot(&state, open_file_slot);

    root
}

pub(super) struct ViewState {
    cwd_provider: CwdProvider,
    model: RefCell<ViewModel>,
    widgets: ViewWidgets,
}

#[derive(Default)]
struct ViewModel {
    current_dir: Option<PathBuf>,
    selected_file: Option<PathBuf>,
    files: Vec<ViewerFile>,
    active_category: FileCategory,
}

struct ViewWidgets {
    list: ListBox,
    count_label: Label,
    preview: PreviewWidgets,
    scope_bar: GtkBox,
}

fn bind_selection(state: &Rc<ViewState>) {
    let state_ref = state.clone();
    state.widgets.list.connect_row_selected(move |_, row| {
        let Some(row) = row else {
            return;
        };
        let file = {
            let visible = visible_files(&state_ref);
            visible.get(row.index() as usize).cloned()
        };
        if let Some(file) = file {
            select_file(&state_ref, &file);
        }
    });
}

fn visible_files(state: &ViewState) -> Vec<ViewerFile> {
    let model = state.model.borrow();
    let active = model.active_category;
    model
        .files
        .iter()
        .filter(|f| active == FileCategory::All || f.category == active)
        .cloned()
        .collect()
}

fn bind_refresh(state: &Rc<ViewState>, refresh_button: &Button) {
    let state_ref = state.clone();
    refresh_button.connect_clicked(move |_| refresh_from_terminal(&state_ref, true));
}

fn watch_terminal_directory(state: &Rc<ViewState>) {
    let state_ref = state.clone();
    glib::timeout_add_local(Duration::from_millis(1200), move || {
        refresh_from_terminal(&state_ref, false);
        glib::ControlFlow::Continue
    });
}

fn watch_open_file_slot(state: &Rc<ViewState>, slot: OpenFileSlot) {
    let state_ref = state.clone();
    glib::timeout_add_local(Duration::from_millis(200), move || {
        let request = slot.borrow_mut().take();
        if let Some(path) = request {
            if let Some(file) = files::viewer_file_for_path(&path) {
                select_file(&state_ref, &file);
            }
        }
        glib::ControlFlow::Continue
    });
}

fn refresh_from_terminal(state: &Rc<ViewState>, force: bool) {
    let dir = current_directory(state).filter(|path| path.is_dir());
    let Some(dir) = dir else {
        state.widgets.count_label.set_text(&format_file_count(0));
        set_empty_preview(&state.widgets.preview, "no terminal context");
        clear_list(&state.widgets.list);
        let mut model = state.model.borrow_mut();
        model.current_dir = None;
        model.selected_file = None;
        model.files.clear();
        return;
    };

    if !force && state.model.borrow().current_dir.as_ref() == Some(&dir) {
        return;
    }

    let files = scan_directory(&dir);
    update_model(state, dir, files);
}

fn current_directory(state: &ViewState) -> Option<PathBuf> {
    state
        .cwd_provider
        .as_ref()()
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
}

fn update_model(state: &Rc<ViewState>, dir: PathBuf, files: Vec<ViewerFile>) {
    let selected = {
        let mut model = state.model.borrow_mut();
        model.current_dir = Some(dir);
        model.files = files;
        // Reset category if old category has no files in new directory.
        let active = model.active_category;
        if active != FileCategory::All
            && !model.files.iter().any(|f| f.category == active)
        {
            model.active_category = FileCategory::All;
        }
        model.selected_file = model
            .selected_file
            .take()
            .filter(|path| model.files.iter().any(|file| &file.path == path));
        model.selected_file.clone()
    };

    rebuild_scope_bar(state);
    rebuild_list(state, selected.as_deref());
    if let Some(path) = selected {
        if let Some(file) = state
            .model
            .borrow()
            .files
            .iter()
            .find(|file| file.path == path)
            .cloned()
        {
            select_file(state, &file);
            return;
        }
    }

    let count = visible_files(state).len();
    state.widgets.count_label.set_text(&format_file_count(count));

    if count == 0 {
        set_empty_preview(&state.widgets.preview, "no supported files in this directory");
        return;
    }

    set_empty_preview(&state.widgets.preview, "select a file to preview");
}

fn rebuild_list(state: &Rc<ViewState>, selected: Option<&std::path::Path>) {
    clear_list(&state.widgets.list);
    let files = visible_files(state);
    state
        .widgets
        .count_label
        .set_text(&format_file_count(files.len()));
    for file in &files {
        let row = file_row(file);
        state.widgets.list.append(&row);
        if selected.is_some_and(|path| path == file.path.as_path()) {
            state.widgets.list.select_row(Some(&row));
        }
    }
}

fn format_file_count(count: usize) -> String {
    match count {
        1 => "1 file".to_string(),
        _ => format!("{count} files"),
    }
}

fn rebuild_scope_bar(state: &Rc<ViewState>) {
    let bar = &state.widgets.scope_bar;
    clear_box(bar);

    let model = state.model.borrow();
    if model.files.is_empty() {
        bar.set_visible(false);
        return;
    }

    // Collect unique categories present in the file list.
    let mut categories: Vec<FileCategory> = Vec::new();
    for file in &model.files {
        if !categories.contains(&file.category) {
            categories.push(file.category);
        }
    }
    categories.sort_by_key(|c| category_order(*c));

    // Hide scope bar if only one category (no point picking).
    if categories.len() <= 1 {
        bar.set_visible(false);
        return;
    }

    bar.set_visible(true);
    let active = model.active_category;
    drop(model);

    // "all" chip.
    let all_chip = scope_chip("all", active == FileCategory::All);
    {
        let state = state.clone();
        all_chip.connect_clicked(move |_| set_active_category(&state, FileCategory::All));
    }
    bar.append(&all_chip);

    for cat in categories {
        let chip = scope_chip(category_label(cat), active == cat);
        {
            let state = state.clone();
            chip.connect_clicked(move |_| set_active_category(&state, cat));
        }
        bar.append(&chip);
    }
}

fn set_active_category(state: &Rc<ViewState>, category: FileCategory) {
    state.model.borrow_mut().active_category = category;
    rebuild_scope_bar(state);
    let selected = state.model.borrow().selected_file.clone();
    rebuild_list(state, selected.as_deref());
    if visible_files(state).is_empty() {
        set_empty_preview(&state.widgets.preview, "no files in this scope");
    }
}

fn scope_chip(label: &str, active: bool) -> Button {
    let btn = Button::with_label(label);
    btn.add_css_class("magma-view-scope-chip");
    if active {
        btn.add_css_class("magma-view-scope-chip-active");
    }
    btn
}

fn clear_box(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
