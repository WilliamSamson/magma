use std::rc::Rc;

use gtk::{glib, prelude::*, Button};
use webkit6::{prelude::*, LoadEvent};

use super::{browser, WebPaneState};

pub(super) fn bind_navigation(
    state: &Rc<WebPaneState>,
    reload_button: &Button,
    home_button: &Button,
    go_button: &Button,
) {
    bind_address(state, go_button);
    bind_history_buttons(state);
    bind_reload(state, reload_button);
    bind_home(state, home_button);
    bind_load_events(state);

    let state_ref = state.clone();
    // Rc clone is required because GTK runs this idle callback after the current build stack returns.
    glib::idle_add_local_once(move || sync_nav_buttons(&state_ref));
}

fn bind_address(state: &Rc<WebPaneState>, go_button: &Button) {
    let address = state.address.clone();
    let submit_state = state.clone();
    // Rc clone is required because entry activation needs owned access to the live WebView after setup returns.
    address.connect_activate(move |entry| open_input(&submit_state, entry.text().as_str()));

    let click_state = state.clone();
    // Rc clone is required because the click callback must outlive this setup scope and drive navigation later.
    go_button.connect_clicked(move |_| open_input(&click_state, click_state.address.text().as_str()));
}

fn bind_history_buttons(state: &Rc<WebPaneState>) {
    let back_button = state.back_button.clone();
    let back_state = state.clone();
    // Rc clone is required because the click callback must own shared access to the live WebView state.
    back_button.connect_clicked(move |_| {
        if back_state.web_view.can_go_back() {
            back_state.web_view.go_back();
        }
    });

    let forward_button = state.forward_button.clone();
    let forward_state = state.clone();
    // Rc clone is required because the click callback must own shared access to the live WebView state.
    forward_button.connect_clicked(move |_| {
        if forward_state.web_view.can_go_forward() {
            forward_state.web_view.go_forward();
        }
    });
}

fn bind_reload(state: &Rc<WebPaneState>, reload_button: &Button) {
    let state_ref = state.clone();
    reload_button.connect_clicked(move |_| {
        state_ref.web_view.reload();
    });
}

fn bind_home(state: &Rc<WebPaneState>, home_button: &Button) {
    let state_ref = state.clone();
    home_button.connect_clicked(move |_| {
        browser::load_home_page(&state_ref);
        sync_nav_buttons(&state_ref);
    });
}

fn bind_load_events(state: &Rc<WebPaneState>) {
    let web_view = state.web_view.clone();
    let load_state = state.clone();
    // Rc clone is required because WebKit emits load updates asynchronously after this setup returns.
    web_view.connect_load_changed(move |web_view, event| {
        let uri = web_view.uri().map(|uri| uri.to_string()).unwrap_or_default();
        let current_title = page_title(web_view, &uri);

        if !uri.is_empty() {
            load_state.address.set_text(&uri);
        }

        let status_text = match event {
            LoadEvent::Started => format!("loading {}", compact_uri(&uri)),
            LoadEvent::Committed => format!("connected {}", compact_uri(&uri)),
            LoadEvent::Finished => current_title.clone(),
            _ => load_state.status.text().to_string(),
        };

        load_state.title.set_text(if current_title.trim().is_empty() {
            "browser"
        } else {
            &current_title
        });
        load_state.status.set_text(&status_text);
        sync_nav_buttons(&load_state);
    });

    let failure_view = state.web_view.clone();
    let failure_state = state.clone();
    // Rc clone is required because load failure callbacks fire independently of the setup scope.
    failure_view.connect_load_failed(move |_web_view, _event, uri, error| {
        failure_state.title.set_text("browser");
        failure_state
            .status
            .set_text(&format!("failed {} · {}", compact_uri(uri), error.message()));
        false
    });

    let notify_view = state.web_view.clone();
    let notify_state = state.clone();
    // Rc clone is required because URI notifications update shared navigation state after navigation changes.
    notify_view.connect_uri_notify(move |_| {
        sync_nav_buttons(&notify_state);
    });
}

fn open_input(state: &WebPaneState, input: &str) {
    let target = browser::resolve_destination(state, input);
    state.address.set_text(&target);
    state.web_view.load_uri(&target);
}

fn sync_nav_buttons(state: &WebPaneState) {
    state.back_button.set_sensitive(state.web_view.can_go_back());
    state
        .forward_button
        .set_sensitive(state.web_view.can_go_forward());
}

fn page_title(web_view: &webkit6::WebView, uri: &str) -> String {
    web_view
        .title()
        .map(|title| title.to_string())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| compact_uri(uri))
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
