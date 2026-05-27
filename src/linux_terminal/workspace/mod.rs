mod ops;
mod switcher;
mod tab_strip;

use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk, prelude::*, Align, Box as GtkBox, Button, EventControllerKey, Notebook, Orientation,
    Overlay, PackType, PolicyType, ScrolledWindow,
};

use super::{
    persist::{self, PaneSnapshot, TabSnapshot, WorkspaceSnapshot},
    settings::Settings,
    terminal::{next_profile, ProfileId, TabView},
};

pub(super) struct WorkspaceView {
    root: GtkBox,
    notebook: Notebook,
    tab_container: GtkBox,
    tab_scroller: ScrolledWindow,
    tabs: Rc<RefCell<Vec<TabView>>>,
    quick_switcher: switcher::QuickSwitcher,
    rename_state: tab_strip::RenameState,
    settings: Rc<RefCell<Settings>>,
    session_bar_host: GtkBox, // GtkBox that acts as the top container host for the active pane's session buttons.
}

impl WorkspaceView {
    pub(super) fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_vexpand(true);
        root.set_focusable(true);

        let tab_container = GtkBox::new(Orientation::Horizontal, 4);
        tab_container.add_css_class("magma-tabs-list");
        tab_container.set_valign(Align::Center);

        let add_button = action_button("list-add-symbolic");
        add_button.add_css_class("magma-add-tab-button");
        add_button.set_halign(Align::End);
        add_button.set_valign(Align::Center);

        let (tab_bar_row, tab_scroller) = tab_bar_row(&tab_container, &add_button);

        // Create the top row container host for active pane sessions
        let session_bar_host = GtkBox::new(Orientation::Horizontal, 0);
        session_bar_host.add_css_class("magma-session-bar-host");

        // Create vertical separator between workspace tabs and sessions
        let separator = gtk::Separator::new(Orientation::Vertical);
        separator.add_css_class("magma-v-separator");

        tab_bar_row.append(&separator);
        tab_bar_row.append(&session_bar_host);

        let notebook = notebook();
        let overlay = Overlay::new();
        overlay.set_hexpand(true);
        overlay.set_vexpand(true);
        let (split_tab, close_tab, profile_tab, actions_box) = actions_box();
        notebook.set_action_widget(&actions_box, PackType::End);
        overlay.set_child(Some(&notebook));

        root.append(&tab_bar_row);

        let tabs = Rc::new(RefCell::new(Vec::new()));
        let quick_switcher = switcher::QuickSwitcher::new(&notebook, &tabs);
        overlay.add_overlay(quick_switcher.widget());
        root.append(&overlay);
        let workspace = Self {
            root,
            notebook,
            tab_container,
            tab_scroller,
            tabs,
            quick_switcher,
            rename_state: Rc::new(RefCell::new(None)),
            settings,
            session_bar_host,
        };

        workspace.restore();
        workspace.bind_actions(add_button, split_tab, close_tab, profile_tab);
        workspace.bind_keyboard_shortcuts();
        workspace.rebuild_tab_strip();

        // Handle initial mounting of the active tab's session bar
        let active_tab = workspace.notebook.current_page().map(|i| i as usize).unwrap_or(0);
        if let Some(tab) = workspace.tabs.borrow().get(active_tab) {
            tab.mount_active_session_bar();
        }

        workspace
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        *self.settings.borrow_mut() = settings.clone();
        for tab in self.tabs.borrow().iter() {
            tab.apply_settings(settings);
        }
    }

    pub(super) fn current_cwd(&self) -> Option<String> {
        self.tabs
            .borrow()
            .get(current_index(&self.notebook))
            .and_then(TabView::current_cwd)
    }

    pub(super) fn current_terminal(&self) -> Option<vte4::Terminal> {
        self.tabs
            .borrow()
            .get(current_index(&self.notebook))
            .and_then(TabView::current_terminal)
    }

    pub(crate) fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            active_tab: current_index(&self.notebook),
            tabs: self.tabs.borrow().iter().map(TabView::to_snapshot).collect(),
        }
    }

    pub(super) fn save(&self) {
        let snapshot = self.snapshot();
        if let Err(error) = persist::save_workspace(&snapshot) {
            eprintln!("workspace save failed: {error}");
        }
    }

    fn restore(&self) {
        let snapshot = persist::load_workspace().ok().flatten().unwrap_or_else(default_snapshot);
        for tab in snapshot.tabs {
            self.append_tab(tab);
        }
        let active_tab = snapshot
            .active_tab
            .min(self.tabs.borrow().len().saturating_sub(1));
        self.notebook.set_current_page(Some(active_tab as u32));

        let tabs = self.tabs.clone();
        gtk::glib::idle_add_local_once(move || {
            if let Some(tab) = tabs.borrow().get(active_tab) {
                tab.restore_focus();
            }
        });
    }

    fn bind_actions(&self, new_tab: Button, split_tab: Button, close_tab: Button, profile_tab: Button) {
        let creation_ctx = TabCreationContext {
            tabs: self.tabs.clone(), // tabs Rc clone is needed to share the reference to the list of tabs.
            notebook: self.notebook.clone(), // notebook clone is needed to refer to the notebook widget.
            tab_container: self.tab_container.clone(), // tab_container clone is needed to rebuild visual tab strip.
            tab_scroller: self.tab_scroller.clone(), // tab_scroller clone is needed to auto-scroll tabs.
            quick_switcher: self.quick_switcher.clone(), // quick_switcher clone is needed to refresh the switcher list.
            rename_state: self.rename_state.clone(), // rename_state clone is needed to preserve renaming context.
            settings: self.settings.clone(), // settings Rc clone is needed to propagate user settings to new tabs.
            session_bar_host: self.session_bar_host.clone(), // session_bar_host GtkBox clone is needed to pass top host reference.
        };

        let ctx = creation_ctx.clone(); // ctx clone is needed to move it into the clicked callback.
        new_tab.connect_clicked(move |_| {
            create_new_tab(&ctx);
        });

        let notebook_split = self.notebook.clone(); // notebook clone is needed inside split callback.
        let tabs_split = self.tabs.clone(); // tabs clone is needed inside split callback to toggle split pane.
        split_tab.connect_clicked(move |_| {
            if let Some(tab) = tabs_split.borrow_mut().get_mut(current_index(&notebook_split)) {
                tab.toggle_split();
            }
        });

        let notebook_close = self.notebook.clone(); // notebook clone is needed inside close callback.
        let tabs_close = self.tabs.clone(); // tabs clone is needed inside close callback to delete tab.
        let tab_container_close = self.tab_container.clone(); // tab_container clone is needed to update tabs visually.
        let rename_state_close = self.rename_state.clone(); // rename_state clone is needed to sync rename states on close.
        let quick_switcher_close = self.quick_switcher.clone(); // quick_switcher clone is needed to refresh active items.
        close_tab.connect_clicked(move |_| {
            let _ = ops::close_tab_at(&tabs_close, &notebook_close, current_index(&notebook_close));
            tab_strip::rebuild_tab_strip(&tab_container_close, &notebook_close, &tabs_close, &rename_state_close);
            quick_switcher_close.refresh();
        });

        let notebook_prof = self.notebook.clone(); // notebook clone is needed inside profile cycle callback.
        let tabs_prof = self.tabs.clone(); // tabs clone is needed inside profile cycle callback to cycle profiles.
        let tab_container_prof = self.tab_container.clone(); // tab_container clone is needed to update tab strip labels.
        let rename_state_prof = self.rename_state.clone(); // rename_state clone is needed to preserve renaming on profile shift.
        let quick_switcher_prof = self.quick_switcher.clone(); // quick_switcher clone is needed to update profile labels in switcher.
        profile_tab.connect_clicked(move |_| {
            if let Some(tab) = tabs_prof.borrow_mut().get_mut(current_index(&notebook_prof)) {
                let next = next_profile(tab.profile_id());
                tab.cycle_profile(next);
            }
            tab_strip::rebuild_tab_strip(&tab_container_prof, &notebook_prof, &tabs_prof, &rename_state_prof);
            quick_switcher_prof.refresh();
        });

        // Rebuild on tab switch so the active indicator and control state always match the page.
        let tabs_switch = self.tabs.clone(); // tabs clone is needed inside switch-page handler to mount the active session bar.
        let tab_container_switch = self.tab_container.clone(); // tab_container clone is needed to rebuild tab strip.
        let tab_scroller_switch = self.tab_scroller.clone(); // tab_scroller clone is needed to focus visual tab container.
        let rename_state_switch = self.rename_state.clone(); // rename_state clone is needed to update rename state on switch.
        let quick_switcher_switch = self.quick_switcher.clone(); // quick_switcher clone is needed to sync switcher selection.
        self.notebook.connect_switch_page(move |notebook, _, page_num| {
            let active = page_num as usize;
            tab_strip::rebuild_tab_strip_at(&tab_container_switch, notebook, &tabs_switch, &rename_state_switch, active);
            tab_strip::reveal_active_tab_at(&tab_container_switch, &tab_scroller_switch, active);
            quick_switcher_switch.refresh();

            // Mount the session bar of the newly active workspace tab:
            if let Some(tab) = tabs_switch.borrow().get(active) {
                tab.mount_active_session_bar();
            }
        });

        // On tab removal: full rebuild needed since widget count changed
        let tabs_rem = self.tabs.clone(); // tabs clone is needed inside page removal handler to rebuild.
        let tab_container_rem = self.tab_container.clone(); // tab_container clone is needed to update list.
        let tab_scroller_rem = self.tab_scroller.clone(); // tab_scroller clone is needed to focus list.
        let rename_state_rem = self.rename_state.clone(); // rename_state clone is needed to sync active tab labels.
        let quick_switcher_rem = self.quick_switcher.clone(); // quick_switcher clone is needed to update switcher list.
        self.notebook.connect_page_removed(move |notebook, _, _| {
            tab_strip::rebuild_tab_strip(&tab_container_rem, notebook, &tabs_rem, &rename_state_rem);
            tab_strip::update_active_tab(&tab_container_rem, notebook);
            tab_strip::reveal_active_tab(&tab_container_rem, &tab_scroller_rem, notebook);
            quick_switcher_rem.refresh();
        });
    }

    fn bind_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);

        let creation_ctx = TabCreationContext {
            tabs: self.tabs.clone(), // tabs Rc clone is needed to share the reference to the list of tabs.
            notebook: self.notebook.clone(), // notebook clone is needed to refer to the notebook widget.
            tab_container: self.tab_container.clone(), // tab_container clone is needed to rebuild visual tab strip.
            tab_scroller: self.tab_scroller.clone(), // tab_scroller clone is needed to auto-scroll tabs.
            quick_switcher: self.quick_switcher.clone(), // quick_switcher clone is needed to refresh the switcher list.
            rename_state: self.rename_state.clone(), // rename_state clone is needed to preserve renaming context.
            settings: self.settings.clone(), // settings Rc clone is needed to propagate user settings to new tabs.
            session_bar_host: self.session_bar_host.clone(), // session_bar_host GtkBox clone is needed to pass top host reference.
        };
        let ctx = creation_ctx.clone(); // ctx clone is needed to move it into the keyboard shortcut callback.

        controller.connect_key_pressed(move |_, key, _, modifiers| {
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
            let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

            if ctx.quick_switcher.is_open() {
                if ctrl && key == gdk::Key::k {
                    ctx.quick_switcher.close();
                    return gtk::glib::Propagation::Stop;
                }
                return gtk::glib::Propagation::Proceed;
            }

            if !ctrl {
                return gtk::glib::Propagation::Proceed;
            }

            match key {
                // Ctrl+T: New tab
                gdk::Key::t if !shift => {
                    create_new_tab(&ctx);
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+W: Close current tab
                gdk::Key::w if alt && !shift => {
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(TabView::close_active_session);
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+W: Close current tab
                gdk::Key::w if !shift && !alt => {
                    let _ = ops::close_tab_at(&ctx.tabs, &ctx.notebook, current_index(&ctx.notebook));
                    tab_strip::rebuild_tab_strip(&ctx.tab_container, &ctx.notebook, &ctx.tabs, &ctx.rename_state);
                    ctx.quick_switcher.refresh();
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Alt+N: New multiplexer session in active pane
                gdk::Key::n if alt && !shift => {
                    if let Some(tab) = ctx.tabs.borrow().get(current_index(&ctx.notebook)) {
                        tab.new_mux_session();
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Tab: Next tab
                gdk::Key::Tab if !shift => {
                    let count = ctx.notebook.n_pages() as usize;
                    if count > 1 {
                        let next = (current_index(&ctx.notebook) + 1) % count;
                        ctx.notebook.set_current_page(Some(next as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Tab: Previous tab
                gdk::Key::Tab if shift => {
                    let count = ctx.notebook.n_pages() as usize;
                    if count > 1 {
                        let current = current_index(&ctx.notebook);
                        let prev = if current == 0 { count - 1 } else { current - 1 };
                        ctx.notebook.set_current_page(Some(prev as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Left: Move tab left
                gdk::Key::Left if shift => {
                    let count = ctx.notebook.n_pages() as usize;
                    let current = current_index(&ctx.notebook);
                    if count > 1 && current > 0 {
                        ops::reorder_tab(&ctx.tabs, &ctx.notebook, current, current - 1);
                        tab_strip::rebuild_tab_strip(&ctx.tab_container, &ctx.notebook, &ctx.tabs, &ctx.rename_state);
                        ctx.notebook.set_current_page(Some((current - 1) as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Right: Move tab right
                gdk::Key::Right if shift => {
                    let count = ctx.notebook.n_pages() as usize;
                    let current = current_index(&ctx.notebook);
                    if count > 1 && current + 1 < count {
                        ops::reorder_tab(&ctx.tabs, &ctx.notebook, current, current + 1);
                        tab_strip::rebuild_tab_strip(&ctx.tab_container, &ctx.notebook, &ctx.tabs, &ctx.rename_state);
                        ctx.notebook.set_current_page(Some((current + 1) as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+R: Rename current tab
                gdk::Key::r if shift => {
                    tab_strip::start_tab_rename(
                        &ctx.tab_container,
                        &ctx.notebook,
                        &ctx.tabs,
                        &ctx.rename_state,
                        current_index(&ctx.notebook),
                    );
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+K: Open quick switcher
                gdk::Key::k if !shift => {
                    ctx.quick_switcher.toggle();
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Alt+Left: Focus left split pane
                gdk::Key::Left if alt => {
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(TabView::focus_left_pane);
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+Alt+Right: Focus right split pane
                gdk::Key::Right if alt => {
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(TabView::focus_right_pane);
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+Alt+PageDown: Next multiplexer session
                gdk::Key::Page_Down if alt => {
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(TabView::focus_next_session);
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+Alt+PageUp: Previous multiplexer session
                gdk::Key::Page_Up if alt => {
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(TabView::focus_previous_session);
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+Alt+1-9: Jump to multiplexer session by number in the active pane
                _ if alt && key.to_unicode().is_some_and(|c| ('1'..='9').contains(&c)) => {
                    let target = (key.to_unicode().unwrap_or('1') as usize) - ('1' as usize);
                    let handled = ctx.tabs
                        .borrow()
                        .get(current_index(&ctx.notebook))
                        .is_some_and(|tab| tab.focus_session(target));
                    if handled {
                        gtk::glib::Propagation::Stop
                    } else {
                        gtk::glib::Propagation::Proceed
                    }
                }
                // Ctrl+1-9: Jump to tab by number
                _ if !alt && key.to_unicode().is_some_and(|c| ('1'..='9').contains(&c)) => {
                    let target = (key.to_unicode().unwrap_or('1') as usize) - ('1' as usize);
                    let count = ctx.notebook.n_pages() as usize;
                    if target < count {
                        ctx.notebook.set_current_page(Some(target as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });

        self.root.add_controller(controller);
    }

    fn append_tab(&self, snapshot: TabSnapshot) {
        append_tab(
            &self.tabs,
            &self.notebook,
            snapshot,
            &self.settings,
            &self.session_bar_host,
        );
    }

    fn rebuild_tab_strip(&self) {
        tab_strip::rebuild_tab_strip(
            &self.tab_container,
            &self.notebook,
            &self.tabs,
            &self.rename_state,
        );
        self.quick_switcher.refresh();
    }
}

#[derive(Clone)]
struct TabCreationContext {
    tabs: Rc<RefCell<Vec<TabView>>>,
    notebook: Notebook,
    tab_container: GtkBox,
    tab_scroller: ScrolledWindow,
    quick_switcher: switcher::QuickSwitcher,
    rename_state: tab_strip::RenameState,
    settings: Rc<RefCell<Settings>>,
    session_bar_host: GtkBox,
}

fn create_new_tab(ctx: &TabCreationContext) {
    let next_index = ctx.tabs.borrow().len() + 1;
    let snapshot = TabSnapshot {
        title: format!("tab {next_index}"),
        profile: ProfileId::Default,
        left_pane: Some(PaneSnapshot::default()),
        right_pane: None,
        split_position: None,
        active_pane: persist::PaneFocus::Left,
    };
    append_tab(&ctx.tabs, &ctx.notebook, snapshot, &ctx.settings, &ctx.session_bar_host);
    ctx.notebook.set_current_page(Some((ctx.tabs.borrow().len().saturating_sub(1)) as u32));
    tab_strip::rebuild_tab_strip(&ctx.tab_container, &ctx.notebook, &ctx.tabs, &ctx.rename_state);
    tab_strip::update_active_tab(&ctx.tab_container, &ctx.notebook);
    tab_strip::reveal_active_tab(&ctx.tab_container, &ctx.tab_scroller, &ctx.notebook);
    ctx.quick_switcher.refresh();
}

fn tab_bar_row(tab_container: &GtkBox, add_button: &Button) -> (GtkBox, ScrolledWindow) {
    let bar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Never)
        .css_classes(["magma-tab-bar-scroller"])
        .build();
    bar_scroll.set_hexpand(true);
    bar_scroll.set_vexpand(false);
    bar_scroll.set_propagate_natural_height(true);
    bar_scroll.set_child(Some(tab_container));

    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.add_css_class("magma-tab-bar-container");
    row.set_valign(Align::Center);
    row.append(&bar_scroll);
    row.append(add_button);
    (row, bar_scroll)
}

fn notebook() -> Notebook {
    let notebook = Notebook::new();
    notebook.set_show_tabs(false);
    notebook.set_show_border(false);
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    notebook.add_css_class("magma-notebook");
    notebook
}

fn actions_box() -> (Button, Button, Button, GtkBox) {
    let actions_box = GtkBox::new(Orientation::Horizontal, 0);
    actions_box.add_css_class("magma-workspace-actions");
    let split_tab = action_button("view-split-left-symbolic");
    let close_tab = action_button("window-close-symbolic");
    let profile_tab = action_button("preferences-system-symbolic");
    for button in [&split_tab, &close_tab, &profile_tab] {
        actions_box.append(button);
    }
    (split_tab, close_tab, profile_tab, actions_box)
}

fn append_tab(
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    snapshot: TabSnapshot,
    settings: &Rc<RefCell<Settings>>, // settings Rc wrapper is cloned inside TabView::new.
    session_bar_host: &GtkBox, // session_bar_host is passed to TabView::new.
) {
    let tab = TabView::new(snapshot, settings.clone(), session_bar_host.clone());
    notebook.append_page(tab.root(), Some(tab.title_label()));
    tabs.borrow_mut().push(tab);
}

fn action_button(icon_name: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .css_classes(["magma-workspace-button"])
        .build()
}

fn current_index(notebook: &Notebook) -> usize {
    notebook.current_page().map(|index| index as usize).unwrap_or(0)
}

fn default_snapshot() -> WorkspaceSnapshot {
    WorkspaceSnapshot {
        active_tab: 0,
        tabs: vec![TabSnapshot {
            title: "tab 1".to_string(),
            profile: ProfileId::Default,
            left_pane: Some(PaneSnapshot::default()),
            right_pane: None,
            split_position: None,
            active_pane: persist::PaneFocus::Left,
        }],
    }
}
