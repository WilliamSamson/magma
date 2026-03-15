use std::rc::Rc;

use gtk::{gio, pango, prelude::*, Box as GtkBox, Button, Label, Orientation, Overflow};
use webkit6::{
    prelude::*, ContextMenuItem, LoadEvent, Settings as WebSettings, UserContentInjectedFrames,
    UserContentManager, UserStyleLevel, UserStyleSheet, WebView,
};

use super::{
    browser,
    persist::{self, WebSnapshot, WebTabSnapshot},
    WebPaneState,
};

pub(in crate::linux_terminal) struct TabInfo {
    pub(super) id: u32,
    pub(super) web_view: WebView,
    pub(super) button: GtkBox,
    pub(super) label: Label,
}

pub(super) fn create_tab(state: &Rc<WebPaneState>) -> usize {
    let id = state.next_tab_id.get();
    state.next_tab_id.set(id + 1);

    let content_manager = build_content_manager();
    let web_settings = WebSettings::new();
    web_settings.set_enable_developer_extras(cfg!(debug_assertions));
    web_settings.set_enable_back_forward_navigation_gestures(true);
    web_settings.set_enable_smooth_scrolling(true);

    let web_view = WebView::builder()
        .web_context(&state.context)
        .network_session(&state.network_session)
        .user_content_manager(&content_manager)
        .settings(&web_settings)
        .zoom_level(0.75)
        .hexpand(true)
        .vexpand(true)
        .build();
    web_view.add_css_class("magma-webview");
    web_view.set_overflow(Overflow::Hidden);

    let label = Label::new(Some("new tab"));
    label.add_css_class("magma-web-tab-label");
    label.set_ellipsize(pango::EllipsizeMode::End);
    label.set_max_width_chars(16);

    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["magma-web-tab-close"])
        .tooltip_text("Close tab")
        .build();

    let tab_button = GtkBox::new(Orientation::Horizontal, 4);
    tab_button.add_css_class("magma-web-tab");
    tab_button.append(&label);
    tab_button.append(&close_button);

    let stack_name = format!("tab-{id}");
    state.content_stack.add_named(&web_view, Some(&stack_name));
    state.tab_bar.append(&tab_button);

    let tab_info = TabInfo {
        id,
        web_view: web_view.clone(),
        button: tab_button.clone(),
        label: label.clone(),
    };

    state.tabs.borrow_mut().push(tab_info);
    let index = state.tabs.borrow().len() - 1;

    bind_tab_click(state, &tab_button, id);
    bind_close_button(state, &close_button, id);
    bind_tab_signals(state, &web_view, id);
    bind_context_menu(state, &web_view, id);
    bind_new_window(state, &web_view);
    bind_find_signals(state, &web_view, id);
    persist_tabs(state);

    index
}

pub(super) fn switch_to_tab(state: &Rc<WebPaneState>, index: usize) {
    let tabs = state.tabs.borrow();
    if index >= tabs.len() {
        return;
    }

    // Remove active class from old tab
    let old_index = state.active_index.get();
    if old_index < tabs.len() {
        tabs[old_index].button.remove_css_class("active");
    }

    state.active_index.set(index);
    tabs[index].button.add_css_class("active");

    let stack_name = format!("tab-{}", tabs[index].id);
    state.content_stack.set_visible_child_name(&stack_name);

    // Sync shared UI from the new active tab
    let web_view = tabs[index].web_view.clone();
    drop(tabs);
    sync_ui_from_view(state, &web_view);
    persist_tabs(state);
}

pub(super) fn close_tab(state: &Rc<WebPaneState>, id: u32) {
    let index = {
        let tabs = state.tabs.borrow();
        match tabs.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return,
        }
    };

    let tab_count = state.tabs.borrow().len();
    if tab_count <= 1 {
        // Last tab: navigate to home instead of closing
        let browser_name = state.settings.borrow().default_browser.clone();
        let (home_uri, _) = browser::home_info(&browser_name);
        let wv = state.tabs.borrow()[0].web_view.clone();
        wv.load_uri(home_uri);
        persist_tabs(state);
        return;
    }

    let tab = state.tabs.borrow_mut().remove(index);
    state.tab_bar.remove(&tab.button);
    state.content_stack.remove(&tab.web_view);

    let active = state.active_index.get();
    if index == active {
        let new_active = index.min(state.tabs.borrow().len() - 1);
        switch_to_tab(state, new_active);
    } else if index < active {
        state.active_index.set(active - 1);
    }

    persist_tabs(state);
}

pub(super) fn restore_tabs(state: &Rc<WebPaneState>) {
    let snapshot = persist::load_snapshot()
        .ok()
        .flatten()
        .filter(|snapshot| !snapshot.tabs.is_empty());

    let Some(snapshot) = snapshot else {
        load_home_tab(state);
        return;
    };

    for tab in snapshot.tabs {
        let idx = create_tab(state);
        let web_view = state.tabs.borrow()[idx].web_view.clone();
        let uri = normalize_snapshot_uri(state, &tab.uri);
        web_view.load_uri(&uri);
    }

    let active = snapshot
        .active_tab
        .min(state.tabs.borrow().len().saturating_sub(1));
    switch_to_tab(state, active);
}

fn sync_ui_from_view(state: &WebPaneState, web_view: &WebView) {
    // Address bar
    if let Some(uri) = web_view.uri() {
        let uri_str = uri.to_string();
        if !uri_str.is_empty() {
            state.address.set_text(&uri_str);
            update_ssl_icon(state, &uri_str);
        }
    }

    // Title in status
    if let Some(title) = web_view.title() {
        let title_str = title.to_string();
        if !title_str.is_empty() {
            state.status.set_text(&title_str);
        } else {
            state.status.set_text("ready");
        }
    } else {
        state.status.set_text("ready");
    }

    // Nav buttons
    state.back_button.set_sensitive(web_view.can_go_back());
    state
        .forward_button
        .set_sensitive(web_view.can_go_forward());

    // Reload/Stop toggle
    let loading = web_view.is_loading();
    state.reload_button.set_visible(!loading);
    state.stop_button.set_visible(loading);

    // Progress bar
    if loading {
        let progress = web_view.estimated_load_progress();
        state.progress_bar.set_fraction(progress);
        state.progress_bar.set_visible(true);
    } else {
        state.progress_bar.set_fraction(0.0);
        state.progress_bar.set_visible(false);
    }

    // Find bar match count
    state.find_matches.set_text("");
}

fn update_ssl_icon(state: &WebPaneState, uri: &str) {
    state.ssl_icon.remove_css_class("secure");
    state.ssl_icon.remove_css_class("insecure");

    if uri.starts_with("https://") {
        state.ssl_icon.set_icon_name(Some("channel-secure-symbolic"));
        state.ssl_icon.add_css_class("secure");
        state.ssl_icon.set_visible(true);
    } else if uri.starts_with("http://") {
        state.ssl_icon.set_icon_name(Some("channel-insecure-symbolic"));
        state.ssl_icon.add_css_class("insecure");
        state.ssl_icon.set_visible(true);
    } else {
        state.ssl_icon.set_visible(false);
    }
}

fn is_active_tab(state: &WebPaneState, id: u32) -> bool {
    let tabs = state.tabs.borrow();
    let idx = state.active_index.get();
    idx < tabs.len() && tabs[idx].id == id
}

fn bind_tab_click(state: &Rc<WebPaneState>, tab_button: &GtkBox, id: u32) {
    let gesture = gtk::GestureClick::new();
    let state_ref = state.clone();
    gesture.connect_released(move |_, _, _, _| {
        let index = {
            let tabs = state_ref.tabs.borrow();
            tabs.iter().position(|t| t.id == id)
        };
        if let Some(index) = index {
            switch_to_tab(&state_ref, index);
        }
    });
    tab_button.add_controller(gesture);
}

fn bind_close_button(state: &Rc<WebPaneState>, close_button: &Button, id: u32) {
    let state_ref = state.clone();
    close_button.connect_clicked(move |_| {
        close_tab(&state_ref, id);
    });
}

fn bind_tab_signals(state: &Rc<WebPaneState>, web_view: &WebView, id: u32) {
    // Load changed
    let load_state = state.clone();
    web_view.connect_load_changed(move |web_view, event| {
        let uri = web_view
            .uri()
            .map(|u| u.to_string())
            .unwrap_or_default();

        if is_active_tab(&load_state, id) {
            if !uri.is_empty() {
                load_state.address.set_text(&uri);
                update_ssl_icon(&load_state, &uri);
            }

            let loading = matches!(event, LoadEvent::Started | LoadEvent::Committed);
            load_state.reload_button.set_visible(!loading);
            load_state.stop_button.set_visible(loading);

            let status_text = match event {
                LoadEvent::Started => format!("loading {}", compact_uri(&uri)),
                LoadEvent::Committed => format!("connected {}", compact_uri(&uri)),
                LoadEvent::Finished => web_view
                    .title()
                    .map(|t| t.to_string())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "ready".to_string()),
                _ => load_state.status.text().to_string(),
            };
            load_state.status.set_text(&status_text);
            load_state
                .back_button
                .set_sensitive(web_view.can_go_back());
            load_state
                .forward_button
                .set_sensitive(web_view.can_go_forward());

            if event == LoadEvent::Finished {
                load_state.progress_bar.set_fraction(0.0);
                load_state.progress_bar.set_visible(false);
            }
        }

        // Always update the tab label
        if event == LoadEvent::Finished {
            if let Some(title) = web_view.title() {
                let t = title.to_string();
                if !t.is_empty() {
                    let tabs = load_state.tabs.borrow();
                    if let Some(tab) = tabs.iter().find(|tab| tab.id == id) {
                        tab.label.set_text(&t);
                    }
                }
            }
        }
    });

    // Load failed
    let fail_state = state.clone();
    web_view.connect_load_failed(move |_web_view, _event, uri, error| {
        if is_active_tab(&fail_state, id) {
            fail_state
                .status
                .set_text(&format!("failed {} · {}", compact_uri(uri), error.message()));
            fail_state.reload_button.set_visible(true);
            fail_state.stop_button.set_visible(false);
            fail_state.progress_bar.set_visible(false);
        }
        false
    });

    // Progress
    let progress_state = state.clone();
    web_view.connect_estimated_load_progress_notify(move |web_view| {
        if is_active_tab(&progress_state, id) {
            let progress = web_view.estimated_load_progress();
            progress_state.progress_bar.set_fraction(progress);
            progress_state.progress_bar.set_visible(true);
        }
    });

    // Title changes
    let title_state = state.clone();
    web_view.connect_title_notify(move |web_view| {
        if let Some(title) = web_view.title() {
            let t = title.to_string();
            if !t.is_empty() {
                // Update tab label
                let tabs = title_state.tabs.borrow();
                if let Some(tab) = tabs.iter().find(|tab| tab.id == id) {
                    tab.label.set_text(&t);
                }

                // Update status bar if active
                if is_active_tab(&title_state, id) && !web_view.is_loading() {
                    title_state.status.set_text(&t);
                }
            }
        }
    });

    // URI changes
    let uri_state = state.clone();
    web_view.connect_uri_notify(move |web_view| {
        if is_active_tab(&uri_state, id) {
            if let Some(uri) = web_view.uri() {
                let uri_str = uri.to_string();
                if !uri_str.is_empty() {
                    uri_state.address.set_text(&uri_str);
                    update_ssl_icon(&uri_state, &uri_str);
                }
            }
            uri_state
                .back_button
                .set_sensitive(web_view.can_go_back());
            uri_state
                .forward_button
                .set_sensitive(web_view.can_go_forward());
        }
        persist_tabs(&uri_state);
    });
}

fn bind_context_menu(state: &Rc<WebPaneState>, web_view: &WebView, _id: u32) {
    let link_uri_holder: Rc<std::cell::RefCell<String>> =
        Rc::new(std::cell::RefCell::new(String::new()));
    let page_uri_holder: Rc<std::cell::RefCell<String>> =
        Rc::new(std::cell::RefCell::new(String::new()));

    // Action group for context menu actions
    let action_group = gio::SimpleActionGroup::new();

    // Open link in new tab
    let open_tab_action = gio::SimpleAction::new("open-in-tab", None);
    let uri_for_open = link_uri_holder.clone();
    let state_for_open = state.clone();
    open_tab_action.connect_activate(move |_, _| {
        let uri = uri_for_open.borrow().clone();
        if !uri.is_empty() {
            let idx = create_tab(&state_for_open);
            switch_to_tab(&state_for_open, idx);
            let wv = state_for_open.tabs.borrow()[idx].web_view.clone();
            wv.load_uri(&uri);
        }
    });
    action_group.add_action(&open_tab_action);

    // Open page in external browser
    let open_external_action = gio::SimpleAction::new("open-external", None);
    let uri_for_ext = page_uri_holder.clone();
    open_external_action.connect_activate(move |_, _| {
        let uri = uri_for_ext.borrow().clone();
        if !uri.is_empty() {
            std::process::Command::new("xdg-open")
                .arg(&uri)
                .spawn()
                .ok();
        }
    });
    action_group.add_action(&open_external_action);

    web_view.insert_action_group("ctx", Some(&action_group));

    let uri_for_menu = link_uri_holder.clone();
    let page_uri_for_menu = page_uri_holder;
    let open_tab_for_menu = open_tab_action;
    let open_ext_for_menu = open_external_action;

    web_view.connect_context_menu(move |web_view, menu, _hit_test| {
        // Store current page URI for "open in external browser"
        if let Some(uri) = web_view.uri() {
            page_uri_for_menu.replace(uri.to_string());
        }

        // If right-clicking a link, add "Open in New Tab"
        if _hit_test.context_is_link() {
            if let Some(link_uri) = _hit_test.link_uri() {
                uri_for_menu.replace(link_uri.to_string());
                menu.append(&ContextMenuItem::new_separator());
                let item =
                    ContextMenuItem::from_gaction(&open_tab_for_menu, "Open in New Tab", None);
                menu.append(&item);
            }
        }

        // Always add "Open in External Browser"
        menu.append(&ContextMenuItem::new_separator());
        let ext_item =
            ContextMenuItem::from_gaction(&open_ext_for_menu, "Open in External Browser", None);
        menu.append(&ext_item);

        false
    });
}

fn bind_new_window(state: &Rc<WebPaneState>, web_view: &WebView) {
    let state_ref = state.clone();
    web_view.connect_create(move |_view, _action| {
        let idx = create_tab(&state_ref);
        switch_to_tab(&state_ref, idx);
        let wv = state_ref.tabs.borrow()[idx].web_view.clone();
        wv.upcast::<gtk::Widget>()
    });
}

fn bind_find_signals(state: &Rc<WebPaneState>, web_view: &WebView, id: u32) {
    let Some(fc) = web_view.find_controller() else {
        return;
    };

    let found_state = state.clone();
    fc.connect_found_text(move |_fc, count| {
        if is_active_tab(&found_state, id) {
            found_state
                .find_matches
                .set_text(&format!("{count} matches"));
        }
    });

    let fail_state = state.clone();
    fc.connect_failed_to_find_text(move |_fc| {
        if is_active_tab(&fail_state, id) {
            fail_state.find_matches.set_text("no matches");
        }
    });
}

pub(super) fn active_web_view(state: &WebPaneState) -> Option<WebView> {
    let tabs = state.tabs.borrow();
    let idx = state.active_index.get();
    tabs.get(idx).map(|t| t.web_view.clone())
}

fn persist_tabs(state: &WebPaneState) {
    let snapshot = WebSnapshot {
        active_tab: state.active_index.get(),
        tabs: state
            .tabs
            .borrow()
            .iter()
            .map(|tab| WebTabSnapshot {
                uri: snapshot_uri(state, &tab.web_view),
            })
            .collect(),
    };

    if let Err(error) = persist::save_snapshot(&snapshot) {
        eprintln!("web snapshot save failed: {error}");
    }
}

fn load_home_tab(state: &Rc<WebPaneState>) {
    let idx = create_tab(state);
    switch_to_tab(state, idx);
    let browser_name = state.settings.borrow().default_browser.clone();
    let (home_uri, label) = browser::home_info(&browser_name);
    state.address.set_text(home_uri);
    state.status.set_text(label);
    let web_view = state.tabs.borrow()[idx].web_view.clone();
    web_view.load_uri(home_uri);
}

fn snapshot_uri(state: &WebPaneState, web_view: &WebView) -> String {
    web_view
        .uri()
        .map(|uri| uri.to_string())
        .filter(|uri| !uri.trim().is_empty())
        .unwrap_or_else(|| normalize_snapshot_uri(state, "").to_string())
}

fn normalize_snapshot_uri(state: &WebPaneState, uri: &str) -> String {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return browser::home_uri(&state.settings.borrow().default_browser).to_string();
    }

    if browser::is_known_home_uri(trimmed) {
        return browser::home_uri(&state.settings.borrow().default_browser).to_string();
    }

    trimmed.to_string()
}

fn compact_uri(uri: &str) -> String {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return "page".to_string();
    }

    trimmed
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

fn build_content_manager() -> UserContentManager {
    let manager = UserContentManager::new();
    manager.add_style_sheet(&dark_mode_stylesheet());
    manager
}

fn dark_mode_stylesheet() -> UserStyleSheet {
    UserStyleSheet::new(
        "
        :root,
        html {
            color-scheme: dark !important;
        }
        ",
        UserContentInjectedFrames::AllFrames,
        UserStyleLevel::User,
        &[],
        &[],
    )
}
