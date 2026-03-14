use gtk::glib;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BrowserProfile {
    label: &'static str,
    home_uri: &'static str,
    search_base: &'static str,
}

const GOOGLE: BrowserProfile = BrowserProfile::new(
    "google",
    "https://www.google.com/",
    "https://www.google.com/search?q=",
);
const DUCKDUCKGO: BrowserProfile = BrowserProfile::new(
    "duckduckgo",
    "https://duckduckgo.com/",
    "https://duckduckgo.com/?q=",
);
const BING: BrowserProfile = BrowserProfile::new(
    "bing",
    "https://www.bing.com/",
    "https://www.bing.com/search?q=",
);
const BRAVE: BrowserProfile = BrowserProfile::new(
    "brave",
    "https://search.brave.com/",
    "https://search.brave.com/search?q=",
);

pub(super) fn resolve_destination(default_browser: &str, input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return home_uri(default_browser).to_string();
    }
    if trimmed.contains(' ') {
        return search_uri(default_browser, trimmed);
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
    search_uri(default_browser, trimmed)
}

pub(super) fn home_info(default_browser: &str) -> (&'static str, &'static str) {
    let profile = browser_profile(default_browser);
    (profile.home_uri, profile.label)
}

pub(super) fn home_uri(default_browser: &str) -> &'static str {
    browser_profile(default_browser).home_uri
}

pub(super) fn is_known_home_uri(uri: &str) -> bool {
    let normalized = trim_trailing_slash(uri);
    [GOOGLE, DUCKDUCKGO, BING, BRAVE]
        .iter()
        .any(|profile| trim_trailing_slash(profile.home_uri) == normalized)
}

fn search_uri(browser: &str, query: &str) -> String {
    let encoded = glib::uri_escape_string(query, None::<&str>, true);
    format!("{}{}", browser_profile(browser).search_base, encoded)
}

fn looks_like_host(input: &str) -> bool {
    input.contains('.') || input.starts_with("localhost") || input.starts_with("127.0.0.1")
}

fn browser_profile(id: &str) -> BrowserProfile {
    match normalize_browser_id(id) {
        "duckduckgo" => DUCKDUCKGO,
        "bing" => BING,
        "brave" => BRAVE,
        _ => GOOGLE,
    }
}

fn normalize_browser_id(id: &str) -> &'static str {
    match id.trim().to_ascii_lowercase().as_str() {
        "duckduckgo" | "duckduck" | "ddg" => "duckduckgo",
        "bing" => "bing",
        "brave" | "bravesearch" | "brave-search" => "brave",
        _ => "google",
    }
}

impl BrowserProfile {
    const fn new(
        label: &'static str,
        home_uri: &'static str,
        search_base: &'static str,
    ) -> Self {
        Self {
            label,
            home_uri,
            search_base,
        }
    }
}

fn trim_trailing_slash(uri: &str) -> &str {
    uri.trim().trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::{home_info, is_known_home_uri, resolve_destination};

    #[test]
    fn every_browser_has_home_and_search_behavior() {
        let cases = [
            ("google", "https://www.google.com/", "https://www.google.com/search?q=rust%20gtk"),
            ("duckduckgo", "https://duckduckgo.com/", "https://duckduckgo.com/?q=rust%20gtk"),
            ("bing", "https://www.bing.com/", "https://www.bing.com/search?q=rust%20gtk"),
            ("brave", "https://search.brave.com/", "https://search.brave.com/search?q=rust%20gtk"),
        ];

        for (id, home, search) in cases {
            assert_eq!(home_info(id).0, home);
            assert_eq!(resolve_destination(id, ""), home);
            assert_eq!(resolve_destination(id, "rust gtk"), search);
        }
    }

    #[test]
    fn browser_aliases_normalize_to_supported_profiles() {
        assert_eq!(home_info("ddg").1, "duckduckgo");
        assert_eq!(home_info("Brave-Search").1, "brave");
        assert_eq!(home_info("unknown").1, "google");
    }

    #[test]
    fn known_home_detection_covers_all_profiles() {
        assert!(is_known_home_uri("https://www.google.com/"));
        assert!(is_known_home_uri("https://duckduckgo.com"));
        assert!(is_known_home_uri("https://www.bing.com/"));
        assert!(is_known_home_uri("https://search.brave.com"));
    }
}
