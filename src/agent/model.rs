use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use super::{
    actions::AgentAction,
    context::{token_budgeted_json, WorkspaceContext},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ModelRequest {
    pub(crate) intent: String,
    pub(crate) context: WorkspaceContext,
}

pub(crate) trait AgentModel: Send + Sync {
    fn plan(&self, request: &ModelRequest) -> Result<Vec<AgentAction>, String>;
}

pub(crate) fn load_model() -> Arc<dyn AgentModel> {
    OpenRouterModel::from_env()
        .map(|model| Arc::new(model) as Arc<dyn AgentModel>)
        .unwrap_or_else(|| Arc::new(NoopModel))
}

const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const PRIMARY_MODEL: &str = "z-ai/glm-4.5-air:free";
const FALLBACK_MODEL: &str = "nvidia/nemotron-3-super-120b-a12b:free";

#[derive(Debug)]
pub(crate) struct OpenRouterModel {
    api_key: String,
    endpoint: String,
    curl_bin: PathBuf,
    rate_limit: Mutex<RateLimitState>,
}

#[derive(Debug)]
struct RateLimitState {
    window_started: Instant,
    requests_in_window: u32,
    last_request: Option<Instant>,
    backoff_until: Option<Instant>,
}

/// Maximum requests allowed in a 60-second window.
const MAX_REQUESTS_PER_WINDOW: u32 = 8;

/// Minimum gap between consecutive requests.
const MIN_REQUEST_GAP_SECS: u64 = 2;

/// Backoff duration when we hit a 429 from the API.
const RATE_LIMIT_BACKOFF_SECS: u64 = 60;

/// Backoff duration when we hit our own local rate limit.
const LOCAL_BACKOFF_SECS: u64 = 30;

impl OpenRouterModel {
    pub(crate) fn from_env() -> Option<Self> {
        let api_key = load_api_key().ok()??;
        Some(Self {
            api_key,
            endpoint: OPENROUTER_ENDPOINT.to_string(),
            curl_bin: PathBuf::from("curl"),
            rate_limit: Mutex::new(RateLimitState {
                window_started: Instant::now(),
                requests_in_window: 0,
                last_request: None,
                backoff_until: None,
            }),
        })
    }

    fn wait_for_slot(&self) -> Result<(), String> {
        let mut state = self
            .rate_limit
            .lock()
            .map_err(|_| "rate-limit lock poisoned".to_string())?;

        if let Some(until) = state.backoff_until {
            let now = Instant::now();
            if until > now {
                let wait = until - now;
                drop(state);
                thread::sleep(wait);
                state = self
                    .rate_limit
                    .lock()
                    .map_err(|_| "rate-limit lock poisoned".to_string())?;
                state.backoff_until = None;
            } else {
                state.backoff_until = None;
            }
        }

        if state.window_started.elapsed() >= Duration::from_secs(60) {
            state.window_started = Instant::now();
            state.requests_in_window = 0;
        }

        if state.requests_in_window >= MAX_REQUESTS_PER_WINDOW {
            state.backoff_until =
                Some(Instant::now() + Duration::from_secs(LOCAL_BACKOFF_SECS));
            return Err(format!(
                "local rate limit ({MAX_REQUESTS_PER_WINDOW}/min); backing off {LOCAL_BACKOFF_SECS}s"
            ));
        }

        if let Some(last) = state.last_request {
            let elapsed = last.elapsed();
            let gap = Duration::from_secs(MIN_REQUEST_GAP_SECS);
            if elapsed < gap {
                let wait = gap - elapsed;
                drop(state);
                thread::sleep(wait);
                state = self
                    .rate_limit
                    .lock()
                    .map_err(|_| "rate-limit lock poisoned".to_string())?;
            }
        }

        state.requests_in_window += 1;
        state.last_request = Some(Instant::now());
        Ok(())
    }

    fn on_rate_limited(&self) {
        if let Ok(mut state) = self.rate_limit.lock() {
            state.backoff_until =
                Some(Instant::now() + Duration::from_secs(RATE_LIMIT_BACKOFF_SECS));
        }
    }

    fn call_model(&self, model: &str, payload: &serde_json::Value) -> Result<String, (u16, String)> {
        let output = Command::new(&self.curl_bin)
            .args([
                "-sS", "--max-time", "20",
                "-X", "POST",
                "-H", "Content-Type: application/json",
                "-H",
            ])
            .arg(format!("Authorization: Bearer {}", self.api_key))
            .args(["-w", "\n%{http_code}", "-d"])
            .arg(payload.to_string())
            .arg(&self.endpoint)
            .output()
            .map_err(|error| (0u16, format!("failed to invoke curl: {error}")))?;

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        let (body, http_code) = match raw.rsplit_once('\n') {
            Some((body, code)) => (body.to_string(), code.trim().parse::<u16>().unwrap_or(0)),
            None => (raw, 0),
        };

        if http_code == 429 {
            return Err((429, format!("{model} rate limited (429)")));
        }

        if http_code >= 400 || !output.status.success() {
            let detail = if stderr.is_empty() {
                body.chars().take(200).collect::<String>()
            } else {
                stderr
            };
            return Err((http_code, format!("{model} error (HTTP {http_code}): {detail}")));
        }

        Ok(body)
    }
}

fn load_api_key() -> Result<Option<String>, String> {
    if let Ok(api_key) = std::env::var("MAGMA_OPENROUTER_API_KEY") {
        if !api_key.trim().is_empty() {
            return Ok(Some(api_key));
        }
    }
    let path = std::env::current_dir()
        .map_err(|error| format!("failed to resolve working directory for .env.agent: {error}"))?
        .join(".env.agent");
    if !path.is_file() {
        return Ok(None);
    }
    ensure_secure_permissions(&path)?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(parse_env_value(&raw, "MAGMA_OPENROUTER_API_KEY"))
}

#[cfg(unix)]
fn ensure_secure_permissions(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(path)
        .map_err(|error| format!("failed to read {} metadata: {error}", path.display()))?
        .permissions()
        .mode();
    if mode & 0o077 != 0 {
        return Err(format!(
            "{} permissions are too open; run `chmod 600 {}`",
            path.display(),
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_secure_permissions(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn parse_env_value(raw: &str, key: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let (name, value) = trimmed.split_once('=')?;
        (name.trim() == key).then(|| value.trim().trim_matches('"').trim_matches('\'').to_string())
    })
}

impl AgentModel for OpenRouterModel {
    fn plan(&self, request: &ModelRequest) -> Result<Vec<AgentAction>, String> {
        self.wait_for_slot()?;
        let context_json = token_budgeted_json(&request.context, 8_000);
        let user_content = format!(
            "## Intent\n{}\n\n## Workspace Context\n{}",
            request.intent, context_json
        );
        let build_payload = |model: &str| {
            serde_json::json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt()},
                    {"role": "user", "content": user_content},
                ],
                "temperature": 0.2,
            })
        };

        // Try primary model, fall back on 429.
        let body = match self.call_model(PRIMARY_MODEL, &build_payload(PRIMARY_MODEL)) {
            Ok(body) => body,
            Err((429, _)) => {
                // Primary rate-limited — try fallback before giving up.
                self.call_model(FALLBACK_MODEL, &build_payload(FALLBACK_MODEL))
                    .map_err(|(code, detail)| {
                        if code == 429 {
                            self.on_rate_limited();
                        }
                        format!("both models failed: {detail}")
                    })?
            }
            Err((_, detail)) => return Err(detail),
        };

        parse_actions(&body)
    }
}

#[derive(Default)]
pub(crate) struct NoopModel;

impl AgentModel for NoopModel {
    fn plan(&self, _request: &ModelRequest) -> Result<Vec<AgentAction>, String> {
        Ok(Vec::new())
    }
}

fn parse_actions(raw: &str) -> Result<Vec<AgentAction>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|error| format!("failed to parse API response: {error}"))?;

    let text = value
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(raw);

    let cleaned = strip_markdown_fences(text);

    // Try strict parse first.
    if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(&cleaned) {
        return Ok(actions);
    }

    // Try as {"actions": [...]}.
    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(&cleaned) {
        if let Some(arr) = wrapper.get("actions") {
            if let Ok(actions) = serde_json::from_value::<Vec<AgentAction>>(arr.clone()) {
                return Ok(actions);
            }
        }
    }

    // Lenient: parse as raw JSON array, skip actions with unknown kinds.
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&cleaned) {
        let actions: Vec<AgentAction> = arr
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        if !actions.is_empty() {
            return Ok(actions);
        }
    }

    // Same lenient parse for {"actions": [...]} wrapper.
    if let Ok(wrapper) = serde_json::from_str::<serde_json::Value>(&cleaned) {
        if let Some(arr) = wrapper.get("actions").and_then(|v| v.as_array()) {
            let actions: Vec<AgentAction> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            if !actions.is_empty() {
                return Ok(actions);
            }
        }
    }

    Err(format!(
        "no parseable actions in response: {}",
        &cleaned[..cleaned.len().min(200)]
    ))
}

fn strip_markdown_fences(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.strip_suffix("```").unwrap_or(rest).trim().to_string()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.strip_suffix("```").unwrap_or(rest).trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn system_prompt() -> &'static str {
    r#"You are Magma, an AI workspace agent in a terminal emulator. Observe context, respond with JSON actions.

RULES: Return ONLY a JSON array of actions. No markdown, no explanation. Each action needs `kind` and `confidence` (0.0–1.0). Use confidence >= 0.90 for clear, safe actions. Never emit destructive commands without confirmation.

ACTIONS:
- surface_message: {"kind":"surface_message","message":"...","confidence":0.95}
- run_command: {"kind":"run_command","command":"...","confidence":0.7} (read-only preferred)
- open_pane: {"kind":"open_pane","pane":"git|logr|web|view|agent","confidence":0.95}
- filter_logr: {"kind":"filter_logr","filter":{"query":"...","levels":["ERROR"]},"confidence":0.9}
- stage_hunk: {"kind":"stage_hunk","hunk":{"file":"...","hunk_index":0,"branch":"..."},"confidence":0.6}
- write_annotation: {"kind":"write_annotation","hunk":{"file":"...","hunk_index":0,"branch":"..."},"note":"...","confidence":0.75}

CONTEXT FIELDS: terminal (cwd, last_lines, last_exit_code, last_command), git (branch, staged/unstaged, conflicted, ahead/behind), logs (entries, levels), active_pane, annotations, memory."#
}
