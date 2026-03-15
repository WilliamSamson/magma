use std::rc::Rc;

use gtk::{gdk, glib, prelude::*, Button, EventControllerKey};
use webkit6::prelude::*;

use super::{browser, tabs, WebPaneState};

#[derive(Clone)]
pub(super) struct NavigationButtons {
    pub(super) home_button: Button,
    pub(super) zoom_out_button: Button,
    pub(super) zoom_reset_button: Button,
    pub(super) go_button: Button,
    pub(super) find_prev: Button,
    pub(super) find_next: Button,
    pub(super) find_close: Button,
}

pub(super) fn bind_navigation(
    state: &Rc<WebPaneState>,
    buttons: NavigationButtons,
) {
    bind_address(state, &buttons.go_button);
    bind_home(state, &buttons.home_button);
    bind_reload_stop(state);
    bind_zoom(state, &buttons.zoom_out_button, &buttons.zoom_reset_button);
    bind_find_bar(state, &buttons.find_prev, &buttons.find_next, &buttons.find_close);
    bind_keyboard(state);
    bind_add_tab(state);

    let state_ref = state.clone();
    glib::idle_add_local_once(move || {
        if let Some(wv) = tabs::active_web_view(&state_ref) {
            state_ref
                .back_button
                .set_sensitive(wv.can_go_back());
            state_ref
                .forward_button
                .set_sensitive(wv.can_go_forward());
        }
    });
}

fn bind_address(state: &Rc<WebPaneState>, go_button: &Button) {
    let address = state.address.clone();
    let submit_state = state.clone();
    address.connect_activate(move |entry| open_input(&submit_state, entry.text().as_str()));

    let click_state = state.clone();
    go_button.connect_clicked(move |_| {
        open_input(&click_state, click_state.address.text().as_str());
    });
}

fn bind_home(state: &Rc<WebPaneState>, home_button: &Button) {
    let state_ref = state.clone();
    home_button.connect_clicked(move |_| {
        let browser_name = state_ref.settings.borrow().default_browser.clone();
        let (home_uri, label) = browser::home_info(&browser_name);
        state_ref.address.set_text(home_uri);
        state_ref.status.set_text(label);
        if let Some(wv) = tabs::active_web_view(&state_ref) {
            wv.load_uri(home_uri);
        }
    });
}

fn bind_reload_stop(state: &Rc<WebPaneState>) {
    let reload_state = state.clone();
    state.reload_button.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&reload_state) {
            wv.reload();
        }
    });

    let stop_state = state.clone();
    state.stop_button.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&stop_state) {
            wv.stop_loading();
        }
        stop_state.reload_button.set_visible(true);
        stop_state.stop_button.set_visible(false);
        stop_state.progress_bar.set_visible(false);
    });
}

fn bind_zoom(state: &Rc<WebPaneState>, zoom_out_button: &Button, zoom_reset_button: &Button) {
    let zoom_out_state = state.clone();
    zoom_out_button.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&zoom_out_state) {
            let next = (wv.zoom_level() - 0.10).max(0.50);
            wv.set_zoom_level(next);
            zoom_out_state
                .status
                .set_text(&format!("zoom {}%", (next * 100.0).round() as i32));
        }
    });

    let zoom_reset_state = state.clone();
    zoom_reset_button.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&zoom_reset_state) {
            wv.set_zoom_level(1.0);
            zoom_reset_state.status.set_text("zoom 100%");
        }
    });
}

fn bind_find_bar(
    state: &Rc<WebPaneState>,
    find_prev: &Button,
    find_next: &Button,
    find_close: &Button,
) {
    // Search as you type
    let search_state = state.clone();
    state.find_entry.connect_changed(move |entry| {
        let text = entry.text();
        if let Some(wv) = tabs::active_web_view(&search_state) {
            if let Some(fc) = wv.find_controller() {
                if text.is_empty() {
                    fc.search_finish();
                    search_state.find_matches.set_text("");
                } else {
                    // CASE_INSENSITIVE (1) | WRAP_AROUND (16)
                    fc.search(&text, 1 | 16, u32::MAX);
                }
            }
        }
    });

    // Enter = next match
    let next_state = state.clone();
    state.find_entry.connect_activate(move |_| {
        if let Some(wv) = tabs::active_web_view(&next_state) {
            if let Some(fc) = wv.find_controller() {
                fc.search_next();
            }
        }
    });

    // Next button
    let next_btn_state = state.clone();
    find_next.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&next_btn_state) {
            if let Some(fc) = wv.find_controller() {
                fc.search_next();
            }
        }
    });

    // Previous button
    let prev_state = state.clone();
    find_prev.connect_clicked(move |_| {
        if let Some(wv) = tabs::active_web_view(&prev_state) {
            if let Some(fc) = wv.find_controller() {
                fc.search_previous();
            }
        }
    });

    // Close button
    let close_state = state.clone();
    find_close.connect_clicked(move |_| {
        close_find_bar(&close_state);
    });
}

fn bind_keyboard(state: &Rc<WebPaneState>) {
    let key_ctrl = EventControllerKey::new();

    let state_ref = state.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);

        if ctrl && keyval == gdk::Key::f {
            // Toggle find bar
            let visible = state_ref.find_bar.is_visible();
            if visible {
                close_find_bar(&state_ref);
            } else {
                state_ref.find_bar.set_visible(true);
                state_ref.find_entry.grab_focus();
            }
            return glib::Propagation::Stop;
        }

        if keyval == gdk::Key::Escape && state_ref.find_bar.is_visible() {
            close_find_bar(&state_ref);
            return glib::Propagation::Stop;
        }

        // Ctrl+T = new tab
        if ctrl && keyval == gdk::Key::t {
            let idx = tabs::create_tab(&state_ref);
            tabs::switch_to_tab(&state_ref, idx);
            let browser_name = state_ref.settings.borrow().default_browser.clone();
            let (home_uri, _) = browser::home_info(&browser_name);
            let wv = state_ref.tabs.borrow()[idx].web_view.clone();
            wv.load_uri(home_uri);
            return glib::Propagation::Stop;
        }

        // Ctrl+W = close tab
        if ctrl && keyval == gdk::Key::w {
            let id = {
                let tabs = state_ref.tabs.borrow();
                let idx = state_ref.active_index.get();
                tabs.get(idx).map(|t| t.id)
            };
            if let Some(id) = id {
                tabs::close_tab(&state_ref, id);
            }
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    state.root.add_controller(key_ctrl);
}

fn bind_add_tab(state: &Rc<WebPaneState>) {
    let state_ref = state.clone();
    state.add_tab_button.connect_clicked(move |_| {
        let idx = tabs::create_tab(&state_ref);
        tabs::switch_to_tab(&state_ref, idx);
        let browser_name = state_ref.settings.borrow().default_browser.clone();
        let (home_uri, _) = browser::home_info(&browser_name);
        let wv = state_ref.tabs.borrow()[idx].web_view.clone();
        wv.load_uri(home_uri);
    });
}

fn open_input(state: &WebPaneState, input: &str) {
    let browser_name = state.settings.borrow().default_browser.clone();
    let target = browser::resolve_destination(&browser_name, input);
    state.address.set_text(&target);
    if let Some(wv) = tabs::active_web_view(state) {
        wv.load_uri(&target);
    }
}

fn close_find_bar(state: &WebPaneState) {
    state.find_bar.set_visible(false);
    state.find_entry.set_text("");
    state.find_matches.set_text("");
    if let Some(wv) = tabs::active_web_view(state) {
        if let Some(fc) = wv.find_controller() {
            fc.search_finish();
        }
    }
}
