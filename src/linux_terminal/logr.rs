use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use gtk::{
    gdk, glib, pango, prelude::*, Box as GtkBox, Button, Entry, EventControllerKey, Label,
    Image, ListBox, ListBoxRow, MenuButton, Orientation, Overflow, Popover, PolicyType,
    ScrolledWindow,
};

use crate::{
    features::logs::{load_source, spawn_file_follower, write_filtered, LogEntry},
    logger,
};

const LOG_EXTENSIONS: &[&str] = &["log", "jsonl", "json", "txt"];
const MAX_SCAN_DEPTH: usize = 3;
const MAX_VISIBLE_ENTRIES: usize = 200;

// ─── Public entry point ─────────────────────────────────────────────

pub(super) fn build_logr_pane() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.add_css_class("magma-logr-root");
    root.set_focusable(true);

    // Tab bar
    let tab_row = build_tab_bar();

    // Header: title + stats + count
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("magma-logr-header");

    let title_block = GtkBox::new(Orientation::Vertical, 2);
    title_block.add_css_class("magma-logr-heading");
    title_block.set_hexpand(true);

    let title = Label::new(Some("logr"));
    title.add_css_class("magma-logr-title");
    title.set_xalign(0.0);

    let count_label = Label::new(Some("0"));
    count_label.add_css_class("magma-logr-count");
    count_label.set_xalign(1.0);

    title_block.append(&title);
    header.append(&title_block);
    header.append(&count_label);

    // File picker
    let popover_list = ListBox::new();
    popover_list.set_selection_mode(gtk::SelectionMode::Single);
    popover_list.add_css_class("magma-logr-popover-list");

    let popover_scroller = ScrolledWindow::new();
    popover_scroller.set_child(Some(&popover_list));
    popover_scroller.set_min_content_height(120);
    popover_scroller.set_max_content_height(280);
    popover_scroller.set_propagate_natural_height(true);
    popover_scroller.set_min_content_width(200);

    let popover = Popover::new();
    popover.set_child(Some(&popover_scroller));
    popover.set_has_arrow(false);
    popover.add_css_class("magma-logr-popover");

    let select_button = MenuButton::builder()
        .label("select log file...")
        .css_classes(["magma-logr-select"])
        .hexpand(true)
        .popover(&popover)
        .build();

    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Rescan for log files")
        .build();

    let picker_row = GtkBox::new(Orientation::Horizontal, 4);
    picker_row.add_css_class("magma-logr-picker");
    let picker_icon = Image::from_icon_name("document-open-symbolic");
    picker_icon.add_css_class("magma-logr-inline-icon");
    picker_row.append(&picker_icon);
    picker_row.append(&select_button);
    picker_row.append(&refresh_button);

    // Controls: play/stop + clear/export
    let controls_row = GtkBox::new(Orientation::Horizontal, 4);
    controls_row.add_css_class("magma-logr-controls");

    let stream_shell = GtkBox::new(Orientation::Horizontal, 6);
    stream_shell.add_css_class("magma-logr-stream-shell");
    let stream_icon = Image::from_icon_name("media-record-symbolic");
    stream_icon.add_css_class("magma-logr-stream-icon");

    let play_button = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Start live stream")
        .sensitive(false)
        .build();

    let stop_button = Button::builder()
        .icon_name("media-playback-stop-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Stop live stream")
        .sensitive(false)
        .visible(false)
        .build();

    let stream_label = Label::new(Some("select a file to stream"));
    stream_label.add_css_class("magma-logr-stream-label");
    stream_label.set_xalign(0.0);
    stream_label.set_hexpand(true);
    stream_shell.append(&stream_icon);
    stream_shell.append(&stream_label);

    let clear_button = Button::builder()
        .icon_name("edit-clear-all-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Clear view (Ctrl+K)")
        .build();

    let export_button = Button::builder()
        .icon_name("document-save-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Export filtered entries")
        .build();

    controls_row.append(&play_button);
    controls_row.append(&stop_button);
    controls_row.append(&stream_shell);
    controls_row.append(&clear_button);
    controls_row.append(&export_button);

    // Filter entry
    let filter_entry = Entry::new();
    filter_entry.add_css_class("magma-logr-filter");
    filter_entry.set_placeholder_text(Some("filter... (Ctrl+L)"));
    filter_entry.set_hexpand(true);
    filter_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("system-search-symbolic"));

    // Log list
    let list = ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("magma-log-list");

    let scroller = ScrolledWindow::new();
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);
    scroller.set_child(Some(&list));

    // Status row
    let status_row = GtkBox::new(Orientation::Horizontal, 4);

    let status = Label::new(Some("idle"));
    status.add_css_class("magma-logr-status");
    status.set_xalign(0.0);
    status.set_hexpand(true);
    status.set_ellipsize(pango::EllipsizeMode::End);

    let jump_button = Button::builder()
        .icon_name("go-bottom-symbolic")
        .css_classes(["magma-logr-icon-btn"])
        .tooltip_text("Jump to bottom")
        .visible(false)
        .build();

    status_row.append(&status);
    status_row.append(&jump_button);

    // Assemble layout
    root.append(&tab_row.row);
    root.append(&header);
    root.append(&picker_row);
    root.append(&controls_row);
    root.append(&filter_entry);
    root.append(&scroller);
    root.append(&status_row);

    // State
    let state = Rc::new(RefCell::new(PaneState {
        tabs: Vec::new(),
        active_tab: 0,
        next_tab_id: 0,
    }));
    let discovered = Rc::new(RefCell::new(Vec::<PathBuf>::new()));

    let view = Rc::new(LogrView {
        root: root.clone(),
        tab_bar: tab_row.tab_bar,
        add_tab_button: tab_row.add_button,
        list: list.clone(),
        count_label,
        status,
        filter_entry: filter_entry.clone(),
        scroller: scroller.clone(),
        select_button: select_button.clone(),
        play_button: play_button.clone(),
        stop_button: stop_button.clone(),
        stream_label: stream_label.clone(),
        clear_button: clear_button.clone(),
        export_button: export_button.clone(),
        state: state.clone(),
        discovered: discovered.clone(),
        popover_list: popover_list.clone(),
        expanded_detail: RefCell::new(None),
    });

    // Create initial tab
    let initial_tab = create_tab(&view);
    switch_to_tab(&view, initial_tab);

    // Initial scan
    populate_popover_list(&popover_list, &discovered);

    // Bind all interactions
    bind_refresh(&view, &refresh_button);
    bind_file_picker(&view, &popover_list, &popover);
    bind_play_stop(&view);
    bind_clear_export(&view);
    bind_filter(&view, &filter_entry);
    bind_tab_add(&view);
    bind_scroll_tracking(&view, &scroller, &jump_button);
    bind_keyboard(&view);

    // Live follower poll
    watch_follower(&view);

    root
}

// ─── State ──────────────────────────────────────────────────────────

struct PaneState {
    tabs: Vec<TabState>,
    active_tab: usize,
    next_tab_id: u32,
}

struct TabState {
    id: u32,
    name: String,
    entries: Vec<LogEntry>,
    follower: Option<Receiver<LogEntry>>,
    selected_path: Option<PathBuf>,
    query: String,
    level_visible: [bool; 6],
    last_status: String,
}

struct LogrView {
    root: GtkBox,
    tab_bar: GtkBox,
    add_tab_button: Button,
    list: ListBox,
    count_label: Label,
    status: Label,
    filter_entry: Entry,
    scroller: ScrolledWindow,
    select_button: MenuButton,
    play_button: Button,
    stop_button: Button,
    stream_label: Label,
    clear_button: Button,
    export_button: Button,
    state: Rc<RefCell<PaneState>>,
    discovered: Rc<RefCell<Vec<PathBuf>>>,
    popover_list: ListBox,
    expanded_detail: RefCell<Option<ExpandedDetail>>,
}

struct ExpandedDetail {
    row: GtkBox,
    revealer: gtk::Revealer,
}

struct TabBarWidgets {
    row: GtkBox,
    tab_bar: GtkBox,
    add_button: Button,
}

fn build_tab_bar() -> TabBarWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("magma-logr-tab-row");
    row.set_hexpand(true);
    row.set_overflow(Overflow::Hidden);

    let scroll = ScrolledWindow::new();
    scroll.add_css_class("magma-logr-tab-scroll");
    scroll.set_hexpand(true);
    scroll.set_vexpand(false);
    scroll.set_policy(PolicyType::Automatic, PolicyType::Never);

    let tab_bar = GtkBox::new(Orientation::Horizontal, 2);
    tab_bar.add_css_class("magma-logr-tabs");
    scroll.set_child(Some(&tab_bar));

    let add_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("New log tab")
        .css_classes(["magma-logr-tab-add"])
        .build();

    row.append(&scroll);
    row.append(&add_button);

    TabBarWidgets {
        row,
        tab_bar,
        add_button,
    }
}

// ─── Tab management ─────────────────────────────────────────────────

fn create_tab(view: &Rc<LogrView>) -> usize {
    let mut state = view.state.borrow_mut();
    let id = state.next_tab_id;
    state.next_tab_id += 1;

    let tab_state = TabState {
        id,
        name: "new".to_string(),
        entries: Vec::new(),
        follower: None,
        selected_path: None,
        query: String::new(),
        level_visible: [true; 6],
        last_status: String::new(),
    };

    let label = Label::new(Some("new"));
    label.add_css_class("magma-logr-tab-label");
    label.set_ellipsize(pango::EllipsizeMode::End);
    label.set_max_width_chars(14);

    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["magma-logr-tab-close"])
        .tooltip_text("Close tab")
        .build();

    let tab_button = GtkBox::new(Orientation::Horizontal, 4);
    tab_button.add_css_class("magma-logr-tab");
    tab_button.append(&label);
    tab_button.append(&close_button);

    view.tab_bar.append(&tab_button);
    state.tabs.push(tab_state);
    let index = state.tabs.len() - 1;
    drop(state);

    // Click to switch
    let view_ref = view.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        let idx = {
            let state = view_ref.state.borrow();
            state.tabs.iter().position(|t| t.id == id)
        };
        if let Some(idx) = idx {
            switch_to_tab(&view_ref, idx);
        }
    });
    tab_button.add_controller(gesture);

    // Close button
    let view_ref = view.clone();
    close_button.connect_clicked(move |_| {
        close_tab(&view_ref, id);
    });

    index
}

fn switch_to_tab(view: &Rc<LogrView>, index: usize) {
    let mut state = view.state.borrow_mut();
    if index >= state.tabs.len() {
        return;
    }

    // Save current tab's query
    let old = state.active_tab;
    if old < state.tabs.len() {
        state.tabs[old].query = view.filter_entry.text().to_string();
    }

    // Remove old active class
    if let Some(btn) = tab_button_at(&view.tab_bar, old) {
        btn.remove_css_class("active");
    }

    state.active_tab = index;

    // Add active class
    if let Some(btn) = tab_button_at(&view.tab_bar, index) {
        btn.add_css_class("active");
    }

    drop(state);
    sync_active_tab_ui(view);
    refresh_view(view);
}

fn close_tab(view: &Rc<LogrView>, id: u32) {
    let index = {
        let state = view.state.borrow();
        match state.tabs.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return,
        }
    };

    let tab_count = view.state.borrow().tabs.len();
    if tab_count <= 1 {
        // Last tab: clear it instead
        {
            let mut state = view.state.borrow_mut();
            let tab = &mut state.tabs[0];
            tab.entries.clear();
            tab.follower = None;
            tab.selected_path = None;
            tab.query.clear();
            tab.level_visible = [true; 6];
            tab.last_status.clear();
            tab.name = "new".to_string();
        }
        if let Some(btn) = tab_button_at(&view.tab_bar, 0) {
            if let Some(lbl) = btn.first_child().and_then(|c| c.downcast::<Label>().ok()) {
                lbl.set_text("new");
            }
        }
        view.filter_entry.set_text("");
        sync_controls_after_stop(view);
        refresh_view(view);
        return;
    }

    // Remove tab button from bar
    if let Some(btn) = tab_button_at(&view.tab_bar, index) {
        view.tab_bar.remove(&btn);
    }

    let active = view.state.borrow().active_tab;
    view.state.borrow_mut().tabs.remove(index);

    if index == active {
        let new_active = index.min(view.state.borrow().tabs.len() - 1);
        view.state.borrow_mut().active_tab = new_active;
        switch_to_tab(view, new_active);
    } else if index < active {
        view.state.borrow_mut().active_tab = active - 1;
    }
}

fn tab_button_at(tab_bar: &GtkBox, index: usize) -> Option<gtk::Widget> {
    let mut child = tab_bar.first_child();
    let mut i = 0;
    while let Some(widget) = child {
        if i == index {
            return Some(widget);
        }
        child = widget.next_sibling();
        i += 1;
    }
    None
}

fn set_tab_label(view: &LogrView, index: usize, text: &str) {
    if let Some(btn) = tab_button_at(&view.tab_bar, index) {
        if let Some(lbl) = btn.first_child().and_then(|c| c.downcast::<Label>().ok()) {
            lbl.set_text(text);
        }
    }
}

// ─── Bindings ───────────────────────────────────────────────────────

fn bind_refresh(view: &Rc<LogrView>, refresh_button: &Button) {
    let popover_list = view.popover_list.clone();
    let discovered = view.discovered.clone();
    refresh_button.connect_clicked(move |_| {
        populate_popover_list(&popover_list, &discovered);
    });
}

fn bind_file_picker(view: &Rc<LogrView>, popover_list: &ListBox, popover: &Popover) {
    let view_ref = view.clone();
    let popover = popover.clone();
    popover_list.connect_row_activated(move |_, row| {
        let idx = row.index() as usize;
        let files = view_ref.discovered.borrow();
        if let Some(path) = files.get(idx) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let mut state = view_ref.state.borrow_mut();
            let active = state.active_tab;
            if active < state.tabs.len() {
                state.tabs[active].selected_path = Some(path.clone());
                state.tabs[active].name = name.to_string();
            }
            drop(state);

            view_ref.select_button.set_label(name);
            view_ref
                .stream_label
                .set_text(&format!("ready: {name}"));
            view_ref.play_button.set_sensitive(true);

            set_tab_label(&view_ref, active_tab_index(&view_ref), name);
        }
        popover.popdown();
    });
}

fn bind_play_stop(view: &Rc<LogrView>) {
    // Play
    let view_ref = view.clone();
    view.play_button.connect_clicked(move |_| {
        let path = {
            let state = view_ref.state.borrow();
            let active = state.active_tab;
            state
                .tabs
                .get(active)
                .and_then(|t| t.selected_path.clone())
        };
        if let Some(path) = path {
            load_file(&path, &view_ref);
        }
    });

    // Stop
    let view_ref = view.clone();
    view.stop_button.connect_clicked(move |_| {
        {
            let mut state = view_ref.state.borrow_mut();
            let active = state.active_tab;
            if let Some(tab) = state.tabs.get_mut(active) {
                tab.follower = None;
                tab.last_status = "stopped".to_string();
            }
        }
        sync_controls_after_stop(&view_ref);
        refresh_view(&view_ref);
    });
}

fn bind_clear_export(view: &Rc<LogrView>) {
    // Clear
    let view_ref = view.clone();
    view.clear_button.connect_clicked(move |_| {
        {
            let mut state = view_ref.state.borrow_mut();
            let active = state.active_tab;
            if let Some(tab) = state.tabs.get_mut(active) {
                tab.entries.clear();
                tab.last_status = "cleared".to_string();
            }
        }
        refresh_view(&view_ref);
        logger::info("logr: view cleared", &[]);
    });

    // Export
    let view_ref = view.clone();
    view.export_button.connect_clicked(move |_| {
        let state = view_ref.state.borrow();
        let active = state.active_tab;
        if let Some(tab) = state.tabs.get(active) {
            let filtered: Vec<&LogEntry> = tab
                .entries
                .iter()
                .filter(|e| matches_filters(e, &tab.query, &tab.level_visible))
                .collect();

            if filtered.is_empty() {
                view_ref.status.set_text("nothing to export");
                return;
            }

            match write_filtered("magma-export.jsonl", &filtered) {
                Ok(count) => {
                    let msg = format!("exported {count} entries → magma-export.jsonl");
                    view_ref.status.set_text(&msg);
                    logger::info("logr: exported", &[("count", &count.to_string())]);
                }
                Err(e) => {
                    view_ref
                        .status
                        .set_text(&format!("export failed: {e}"));
                    logger::error("logr: export failed", &[("error", &e.to_string())]);
                }
            }
        }
    });
}

fn bind_filter(view: &Rc<LogrView>, filter_entry: &Entry) {
    let view_ref = view.clone();
    filter_entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        {
            let mut state = view_ref.state.borrow_mut();
            let active = state.active_tab;
            if let Some(tab) = state.tabs.get_mut(active) {
                tab.query = text;
            }
        }
        refresh_view(&view_ref);
    });
}

fn bind_tab_add(view: &Rc<LogrView>) {
    let view_ref = view.clone();
    view.add_tab_button.connect_clicked(move |_| {
        let idx = create_tab(&view_ref);
        switch_to_tab(&view_ref, idx);
    });
}

fn bind_scroll_tracking(_view: &Rc<LogrView>, scroller: &ScrolledWindow, jump_button: &Button) {
    let jump_ref = jump_button.clone();
    let adj = scroller.vadjustment();
    adj.connect_value_changed(move |adj| {
        let at_bottom = adj.value() + adj.page_size() >= adj.upper() - 50.0;
        jump_ref.set_visible(!at_bottom);
    });

    let scroller_ref = scroller.clone();
    jump_button.connect_clicked(move |btn| {
        scroll_to_bottom(&scroller_ref);
        btn.set_visible(false);
    });
}

fn bind_keyboard(view: &Rc<LogrView>) {
    let key_ctrl = EventControllerKey::new();
    let view_ref = view.clone();

    key_ctrl.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);

        // Ctrl+L → focus filter
        if ctrl && keyval == gdk::Key::l {
            view_ref.filter_entry.grab_focus();
            return glib::Propagation::Stop;
        }

        // Ctrl+K → clear view
        if ctrl && keyval == gdk::Key::k {
            {
                let mut state = view_ref.state.borrow_mut();
                let active = state.active_tab;
                if let Some(tab) = state.tabs.get_mut(active) {
                    tab.entries.clear();
                    tab.last_status = "cleared".to_string();
                }
            }
            refresh_view(&view_ref);
            return glib::Propagation::Stop;
        }

        // Ctrl+T → new tab
        if ctrl && keyval == gdk::Key::t {
            let idx = create_tab(&view_ref);
            switch_to_tab(&view_ref, idx);
            return glib::Propagation::Stop;
        }

        // Ctrl+W → close tab
        if ctrl && keyval == gdk::Key::w {
            let id = {
                let state = view_ref.state.borrow();
                state
                    .tabs
                    .get(state.active_tab)
                    .map(|t| t.id)
            };
            if let Some(id) = id {
                close_tab(&view_ref, id);
            }
            return glib::Propagation::Stop;
        }

        // Escape → unfocus filter
        if keyval == gdk::Key::Escape {
            view_ref.root.grab_focus();
            return glib::Propagation::Stop;
        }

        // / → focus filter (only when filter is not focused)
        if keyval == gdk::Key::slash && !view_ref.filter_entry.has_focus() {
            view_ref.filter_entry.grab_focus();
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    view.root.add_controller(key_ctrl);
}

// ─── File discovery ─────────────────────────────────────────────────

fn populate_popover_list(popover_list: &ListBox, discovered: &Rc<RefCell<Vec<PathBuf>>>) {
    while let Some(child) = popover_list.first_child() {
        popover_list.remove(&child);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut files = Vec::new();
    walk_for_logs(&cwd, 0, &mut files);
    files.sort();

    for file in &files {
        let display = file
            .strip_prefix(&cwd)
            .unwrap_or(file)
            .display()
            .to_string();

        let label = Label::new(Some(&display));
        label.set_xalign(0.0);
        label.add_css_class("magma-logr-popover-item");

        let row = ListBoxRow::new();
        row.set_child(Some(&label));
        row.add_css_class("magma-logr-popover-row");
        popover_list.append(&row);
    }

    *discovered.borrow_mut() = files;
}

fn walk_for_logs(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
        }

        if path.is_dir() {
            walk_for_logs(&path, depth + 1, out);
        } else if is_log_file(&path) {
            out.push(path);
        }
    }
}

fn is_log_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| LOG_EXTENSIONS.contains(&ext))
}

// ─── Loading ────────────────────────────────────────────────────────

fn load_file(path: &Path, view: &Rc<LogrView>) {
    let path_str = path.display().to_string();
    logger::info("logr: loading file", &[("path", &path_str)]);
    match load_source(Some(path_str.clone())) {
        Ok(source) => {
            let follower = source.follow_config.map(spawn_file_follower);
            let loaded = source.entries.len();
            let mode = if follower.is_some() { "live" } else { "static" };
            logger::info("logr: file loaded", &[
                ("entries", &loaded.to_string()),
                ("mode", mode),
            ]);
            let query = view.filter_entry.text().to_string();
            {
                let mut state = view.state.borrow_mut();
                let active = state.active_tab;
                if let Some(tab) = state.tabs.get_mut(active) {
                    tab.entries = source.entries;
                    tab.follower = follower;
                    tab.query = query;
                    tab.last_status = format!("{loaded} entries ({mode})");
                }
            }
            sync_active_tab_ui(view);
            refresh_view(view);
        }
        Err(error) => {
            logger::error("logr: file load failed", &[("error", &error.to_string())]);
            {
                let mut state = view.state.borrow_mut();
                let active = state.active_tab;
                if let Some(tab) = state.tabs.get_mut(active) {
                    tab.last_status = format!("error: {error}");
                }
            }
            sync_active_tab_ui(view);
            refresh_view(view);
        }
    }
}

// ─── Live follower ──────────────────────────────────────────────────

fn watch_follower(view: &Rc<LogrView>) {
    let view = view.clone();
    glib::timeout_add_local(Duration::from_millis(250), move || {
        let changed = drain_all_tabs(&view.state);
        if changed {
            sync_active_tab_ui(&view);
            refresh_view(&view);
            // Auto-scroll if near bottom
            if should_auto_scroll(&view.scroller) {
                let scroller = view.scroller.clone();
                glib::idle_add_local_once(move || scroll_to_bottom(&scroller));
            }
        }
        glib::ControlFlow::Continue
    });
}

fn drain_all_tabs(state: &Rc<RefCell<PaneState>>) -> bool {
    let mut changed = false;
    let mut s = state.borrow_mut();
    for tab in &mut s.tabs {
        let mut new_entries = Vec::new();
        let mut stopped = false;

        if let Some(receiver) = tab.follower.as_mut() {
            loop {
                match receiver.try_recv() {
                    Ok(entry) => new_entries.push(entry),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        stopped = true;
                        break;
                    }
                }
            }
        }

        if !new_entries.is_empty() || stopped {
            tab.entries.append(&mut new_entries);
            if stopped {
                tab.follower = None;
                tab.last_status = "follow ended".to_string();
            }
            changed = true;
        }
    }
    changed
}

fn should_auto_scroll(scroller: &ScrolledWindow) -> bool {
    let adj = scroller.vadjustment();
    adj.value() + adj.page_size() >= adj.upper() - 50.0
}

fn scroll_to_bottom(scroller: &ScrolledWindow) {
    let adj = scroller.vadjustment();
    adj.set_value(adj.upper() - adj.page_size());
}

// ─── View rendering ─────────────────────────────────────────────────

fn refresh_view(view: &Rc<LogrView>) {
    collapse_expanded_detail(view);
    clear_list(&view.list);

    let state = view.state.borrow();
    let active = state.active_tab;
    let Some(tab) = state.tabs.get(active) else {
        return;
    };

    // Cloning filtered entries releases the state borrow before GTK row construction, which avoids holding UI work against RefCell state.
    let filtered: Vec<LogEntry> = tab
        .entries
        .iter()
        .filter(|e| matches_filters(e, &tab.query, &tab.level_visible))
        .cloned()
        .collect();
    let shown = filtered.len();
    let total = tab.entries.len();
    let start = shown.saturating_sub(MAX_VISIBLE_ENTRIES);

    view.count_label.set_text(&format_count_summary(shown, total));
    view.status.set_text(if tab.last_status.is_empty() {
        "idle"
    } else {
        &tab.last_status
    });

    if shown == 0 {
        let msg = if total == 0 {
            "no logs loaded"
        } else {
            "no matches"
        };
        let lbl = Label::new(Some(msg));
        lbl.add_css_class("magma-logr-empty");
        lbl.set_xalign(0.0);
        view.list.append(&lbl);
        return;
    }

    let query = tab.query.clone();
    let selected_path = tab.selected_path.clone();
    drop(state);

    for entry in filtered.into_iter().skip(start) {
        view.list
            .append(&entry_row(&entry, &query, &selected_path, view));
    }
}

fn entry_row(
    entry: &LogEntry,
    query: &str,
    selected_path: &Option<PathBuf>,
    view: &Rc<LogrView>,
) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("magma-log-entry-container");

    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("magma-log-entry");

    let level = entry.level_label().to_lowercase();
    row.add_css_class(&format!("log-{level}"));

    // Line Number
    let line_num = Label::new(Some(&format!("{:>4}", entry.line_number())));
    line_num.add_css_class("magma-log-line-number");

    // Timestamp
    let ts_label = if let Some(ts) = entry.timestamp() {
        let formatted = format_timestamp(ts);
        let lbl = Label::new(Some(&formatted));
        lbl.add_css_class("magma-log-timestamp");
        lbl.set_tooltip_text(Some(ts));
        Some(lbl)
    } else {
        None
    };

    // Level badge
    let badge = Label::new(Some(
        &level
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string(),
    ));
    badge.add_css_class("log-level-dot");

    // Body with search highlighting
    let body_text = entry.message();
    let body = Label::new(None);
    if query.is_empty() {
        body.set_text(body_text);
    } else {
        body.set_markup(&highlight_text(body_text, query));
    }
    body.set_xalign(0.0);
    body.set_ellipsize(pango::EllipsizeMode::End);
    body.set_hexpand(true);
    body.add_css_class("log-body");

    // Fields summary (dimmed, after message)
    let fields = entry.fields_summary();
    if !fields.is_empty() {
        let fields_label = Label::new(None);
        fields_label.add_css_class("magma-log-fields");
        if query.is_empty() {
            fields_label.set_text(fields);
        } else {
            fields_label.set_markup(&highlight_text(fields, query));
        }
        fields_label.set_use_markup(true);
        fields_label.set_ellipsize(pango::EllipsizeMode::End);
        row.append(&fields_label);
        body.set_tooltip_text(Some(&format!("{body_text} {fields}")));
    }

    let copy_btn = Button::builder()
        .icon_name("edit-copy-symbolic")
        .css_classes(["magma-log-copy-btn"])
        .tooltip_text("Copy log line")
        .build();

    let delete_btn = Button::builder()
        .icon_name("edit-delete-symbolic")
        .css_classes(["magma-log-delete-btn"])
        .tooltip_text("Delete log line")
        .build();

    let raw_content = entry.raw_line().to_string();
    let line_num_val = entry.line_number();

    // Copy logic
    {
        let copy_btn_ref = copy_btn.clone();
        let raw = raw_content.clone();
        copy_btn.connect_clicked(move |_| {
            let Some(display) = gdk::Display::default() else {
                logger::error("logr: clipboard unavailable", &[]);
                return;
            };
            display.clipboard().set_text(&raw);
            logger::info(
                "logr: line copied to clipboard",
                &[("length", &raw.len().to_string())],
            );
            copy_btn_ref.set_icon_name("emblem-ok-symbolic");
            let btn = copy_btn_ref.clone();
            glib::timeout_add_local(Duration::from_millis(1200), move || {
                btn.set_icon_name("edit-copy-symbolic");
                glib::ControlFlow::Break
            });
        });
    }

    // Delete logic
    {
        let view_ref = view.clone();
        let path = selected_path.clone();
        delete_btn.connect_clicked(move |_| {
            if let Some(path) = &path {
                match crate::features::logs::remove_line_at(path, line_num_val) {
                    Ok(_) => {
                        logger::info(
                            "logr: line deleted from file",
                            &[("line", &line_num_val.to_string())],
                        );
                        load_file(path, &view_ref);
                        view_ref
                            .status
                            .set_text(&format!("line {line_num_val} deleted"));
                    }
                    Err(e) => {
                        logger::error(
                            "logr: delete failed",
                            &[("error", &e.to_string())],
                        );
                    }
                }
            }
        });
    }

    row.append(&line_num);
    if let Some(ts) = ts_label {
        row.append(&ts);
    }
    row.append(&badge);
    row.append(&body);
    row.append(&copy_btn);
    row.append(&delete_btn);

    // Expandable details with structured JSON rendering
    let details_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .transition_duration(200)
        .build();

    let details_label = Label::new(None);
    details_label.set_markup(&format_details(&raw_content));
    details_label.add_css_class("magma-log-details");
    details_label.set_selectable(true);
    details_label.set_wrap(true);
    details_label.set_wrap_mode(pango::WrapMode::WordChar);
    details_label.set_xalign(0.0);
    details_label.set_margin_start(42);
    details_label.set_margin_end(16);
    details_label.set_margin_bottom(8);

    details_revealer.set_child(Some(&details_label));

    container.append(&row);
    container.append(&details_revealer);

    // Toggle expansion on row click
    let gesture = gtk::GestureClick::new();
    let revealer_ref = details_revealer.clone();
    let row_ref = row.clone();
    let view_ref = view.clone();
    gesture.connect_released(move |_, _, _, _| {
        toggle_expanded_detail(&view_ref, &row_ref, &revealer_ref);
    });
    row.add_controller(gesture);

    container
}

fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

// ─── Filtering ──────────────────────────────────────────────────────

fn matches_filters(entry: &LogEntry, query: &str, level_visible: &[bool; 6]) -> bool {
    level_visible[entry.level().index()] && entry.matches_query(query)
}

// ─── Formatting helpers ─────────────────────────────────────────────

fn format_timestamp(ts: &str) -> String {
    // Epoch integer
    if let Ok(epoch) = ts.parse::<i64>() {
        return format_epoch(epoch);
    }
    // Epoch float
    if let Ok(epoch) = ts.parse::<f64>() {
        return format_epoch(epoch as i64);
    }
    // ISO 8601: extract time portion
    if let Some(t) = ts.find('T') {
        let rest = &ts[t + 1..];
        let end = rest
            .find(|c: char| c == 'Z' || c == '+' || (c == '-' && rest.len() > 8))
            .unwrap_or(rest.len());
        let time = &rest[..end];
        // Trim to HH:MM:SS
        if time.len() > 8 {
            return time[..8].to_string();
        }
        return time.to_string();
    }
    ts.to_string()
}

fn format_epoch(epoch: i64) -> String {
    let secs = epoch.rem_euclid(86400);
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

fn format_details(raw: &str) -> String {
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw)
    else {
        return glib::markup_escape_text(raw).to_string();
    };

    let mut lines = Vec::new();
    for (key, value) in &map {
        let val_str = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
        };
        lines.push(format!(
            "<span alpha=\"50%\">{}</span>  {}",
            glib::markup_escape_text(key),
            glib::markup_escape_text(&val_str),
        ));
    }

    if lines.is_empty() {
        return glib::markup_escape_text(raw).to_string();
    }
    lines.join("\n")
}

fn highlight_text(text: &str, query: &str) -> String {
    if query.is_empty() {
        return glib::markup_escape_text(text).to_string();
    }

    let text_lower = text.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();

    let mut result = String::new();
    let mut last_end = 0;

    for (start, matched) in text_lower.match_indices(&query_lower) {
        let end = start + matched.len();
        result.push_str(&glib::markup_escape_text(&text[last_end..start]));
        result.push_str("<span background=\"#ff4d4d\" foreground=\"#0b0b0b\">");
        result.push_str(&glib::markup_escape_text(&text[start..end]));
        result.push_str("</span>");
        last_end = end;
    }
    result.push_str(&glib::markup_escape_text(&text[last_end..]));

    result
}

// ─── Helpers ────────────────────────────────────────────────────────

fn active_tab_index(view: &LogrView) -> usize {
    view.state.borrow().active_tab
}

fn format_count_summary(shown: usize, total: usize) -> String {
    match (shown, total) {
        (0, 0) => "no entries".to_string(),
        (shown, total) if shown == total => format!("{total} entries"),
        (shown, total) => format!("{shown} of {total}"),
    }
}

fn sync_active_tab_ui(view: &Rc<LogrView>) {
    let (select_label, streaming, play_sensitive, query, stream_label) = {
        let state = view.state.borrow();
        let Some(tab) = state.tabs.get(state.active_tab) else {
            return;
        };

        (
            tab.selected_path
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("select log file...")
                .to_string(),
            tab.follower.is_some(),
            tab.selected_path.is_some(),
            tab.query.clone(),
            stream_label_text(tab),
        )
    };

    view.select_button.set_label(&select_label);
    view.play_button.set_visible(!streaming);
    view.play_button.set_sensitive(play_sensitive);
    view.stop_button.set_visible(streaming);
    view.stop_button.set_sensitive(streaming);

    if view.filter_entry.text().as_str() != query {
        view.filter_entry.set_text(&query);
    }

    view.stream_label.set_text(&stream_label);
}

fn stream_label_text(tab: &TabState) -> String {
    if tab.follower.is_some() {
        return format!("streaming: {}", tab.name);
    }

    if tab.selected_path.is_some() {
        if tab.last_status == "stopped" {
            return format!("stopped: {}", tab.name);
        }
        return format!("ready: {}", tab.name);
    }

    "select a file to stream".to_string()
}

fn sync_controls_after_stop(view: &Rc<LogrView>) {
    sync_active_tab_ui(view);
}

fn collapse_expanded_detail(view: &LogrView) {
    if let Some(detail) = view.expanded_detail.borrow_mut().take() {
        detail.revealer.set_reveal_child(false);
        detail.row.remove_css_class("expanded");
    }
}

fn toggle_expanded_detail(view: &Rc<LogrView>, row: &GtkBox, revealer: &gtk::Revealer) {
    let mut expanded_detail = view.expanded_detail.borrow_mut();
    let reopen_same_row = expanded_detail
        .as_ref()
        .is_some_and(|detail| detail.row == *row);

    if let Some(detail) = expanded_detail.take() {
        detail.revealer.set_reveal_child(false);
        detail.row.remove_css_class("expanded");
    }

    if reopen_same_row {
        return;
    }

    revealer.set_reveal_child(true);
    row.add_css_class("expanded");
    // GTK widget clone shares the same object handle, so this tracks the live expanded row without duplicating widgets.
    *expanded_detail = Some(ExpandedDetail {
        row: row.clone(),
        revealer: revealer.clone(),
    });
}
