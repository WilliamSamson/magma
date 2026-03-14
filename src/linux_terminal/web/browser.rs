use gtk::prelude::*;
use gtk::glib;
use webkit6::prelude::*;

use super::WebPaneState;

pub(super) fn load_home_page(state: &WebPaneState) {
    let browser = browser_profile(&state.settings.borrow().default_browser);
    state.address.set_text(browser.home_uri);
    state.title.set_text("browser");
    state.status.set_text(browser.label);
    state.web_view.load_uri(browser.home_uri);
}

pub(super) fn resolve_destination(state: &WebPaneState, input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return browser_profile(&state.settings.borrow().default_browser)
            .home_uri
            .to_string();
    }
    if trimmed.contains(' ') {
        return search_uri(&state.settings.borrow().default_browser, trimmed);
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    if looks_like_host(trimmed) {
        if trimmed.starts_with("localhost") || trimmed.starts_with("127.0.0.1") {
            return format!("http://{trimmed}");
        }
        return format!("https://{trimmed}");
    }
    search_uri(&state.settings.borrow().default_browser, trimmed)
}

fn search_uri(browser: &str, query: &str) -> String {
    let encoded = glib::uri_escape_string(query, None::<&str>, true);
    format!("{}{}", browser_profile(browser).search_base, encoded)
}

fn looks_like_host(input: &str) -> bool {
    input.contains('.') || input.starts_with("localhost") || input.starts_with("127.0.0.1")
}

fn browser_profile(id: &str) -> BrowserProfile {
    match id {
        "duckduckgo" => BrowserProfile::new("duckduckgo", "https://duckduckgo.com/", "https://duckduckgo.com/?q="),
        "bing" => BrowserProfile::new("bing", "https://www.bing.com/", "https://www.bing.com/search?q="),
        "brave" => BrowserProfile::new("brave", "https://search.brave.com/", "https://search.brave.com/search?q="),
        _ => BrowserProfile::new("google", "https://www.google.com/", "https://www.google.com/search?q="),
    }
}

struct BrowserProfile {
    label: &'static str,
    home_uri: &'static str,
    search_base: &'static str,
}

impl BrowserProfile {
    fn new(label: &'static str, home_uri: &'static str, search_base: &'static str) -> Self {
        Self {
            label,
            home_uri,
            search_base,
        }
    }
}
