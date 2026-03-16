use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::Write,
    sync::{
        mpsc::{self, RecvTimeoutError, Sender},
        Arc, Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use super::{
    actions::AgentAction,
    context::{build_workspace_context, token_budgeted_json},
    data_root,
    effects::{execute_side_effect, UiEffect},
    ensure_parent,
    model::{AgentModel, ModelRequest},
    observer::{self, WorkspaceEvent},
};

/// A single entry in the agent's conversation log, visible in the pane UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LogEntry {
    pub(crate) role: LogRole,
    pub(crate) text: String,
    pub(crate) timestamp_ms: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogRole {
    User,
    Agent,
    System,
    Action,
}

/// Snapshot of the runtime state readable from the GTK thread.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct UiUpdate {
    pub(crate) status: String,
    pub(crate) message: Option<String>,
    pub(crate) pending_action: Option<AgentAction>,
    pub(crate) dry_run: Option<String>,
}

/// Full state including conversation history.
#[derive(Clone, Debug, Default)]
pub(crate) struct RuntimeState {
    pub(crate) ui: UiUpdate,
    pub(crate) log: VecDeque<LogEntry>,
}

const MAX_LOG_ENTRIES: usize = 200;

/// Minimum seconds between model calls triggered by observer events.
/// User prompts bypass this throttle.
const EVENT_THROTTLE_SECS: u64 = 60;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExecutorConfig {
    pub(crate) confidence_threshold: f32,
    pub(crate) passive_mode: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.90,
            passive_mode: false,
        }
    }
}

enum RuntimeCommand {
    Prompt(String),
    Decision(bool),
    SetPassive(bool),
}

pub(crate) struct AgentRuntimeHandle {
    tx: Sender<RuntimeCommand>,
    // Arc<Mutex<...>> keeps the latest runtime state readable from GTK polling
    // while the worker thread mutates it safely.
    state: Arc<Mutex<RuntimeState>>,
    // Arc<Mutex<VecDeque<_>>> is a simple cross-thread queue for UI-thread-only
    // mutations like opening panes.
    ui_effects: Arc<Mutex<VecDeque<UiEffect>>>,
}

static RUNTIME: OnceLock<AgentRuntimeHandle> = OnceLock::new();

impl AgentRuntimeHandle {
    pub(crate) fn shared(config: ExecutorConfig) -> &'static Self {
        RUNTIME.get_or_init(|| spawn(config))
    }

    pub(crate) fn snapshot(&self) -> UiUpdate {
        self.state
            .lock()
            .map(|state| state.ui.clone())
            .unwrap_or_default()
    }

    pub(crate) fn log_entries(&self) -> Vec<LogEntry> {
        self.state
            .lock()
            .map(|state| state.log.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns how many log entries exist (for change detection without cloning).
    pub(crate) fn log_len(&self) -> usize {
        self.state.lock().map(|s| s.log.len()).unwrap_or(0)
    }

    pub(crate) fn submit_prompt(&self, prompt: String) {
        let _ = self.tx.send(RuntimeCommand::Prompt(prompt));
    }

    pub(crate) fn respond(&self, accept: bool) {
        let _ = self.tx.send(RuntimeCommand::Decision(accept));
    }

    pub(crate) fn set_passive_mode(&self, passive: bool) {
        let _ = self.tx.send(RuntimeCommand::SetPassive(passive));
    }

    pub(crate) fn drain_ui_effects(&self) -> Vec<UiEffect> {
        let mut guard = match self.ui_effects.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        guard.drain(..).collect()
    }
}

fn spawn(config: ExecutorConfig) -> AgentRuntimeHandle {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(Mutex::new(RuntimeState {
        ui: UiUpdate {
            status: "Watching".to_string(),
            message: Some("observer online".to_string()),
            pending_action: None,
            dry_run: None,
        },
        log: VecDeque::new(),
    }));
    let ui_effects = Arc::new(Mutex::new(VecDeque::new()));
    let state_ref = state.clone();
    let effects_ref = ui_effects.clone();
    thread::spawn(move || {
        let event_rx = observer::spawn(observer::ObserverConfig::default());
        let model = super::model::load_model();
        let mut passive_mode = config.passive_mode;
        let mut pending_action: Option<AgentAction> = None;

        // Track when the last event-triggered model call happened so we can
        // throttle observer-driven calls globally.
        let mut last_event_call = Instant::now() - Duration::from_secs(EVENT_THROTTLE_SECS);
        // Buffer of events waiting for the throttle to expire.
        let mut event_buffer: Vec<WorkspaceEvent> = Vec::new();

        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(RuntimeCommand::Prompt(prompt)) => {
                    push_log(
                        &state_ref,
                        LogEntry {
                            role: LogRole::User,
                            text: prompt.clone(),
                            timestamp_ms: now_ms(),
                        },
                    );
                    // User prompts always go through immediately.
                    process_request(
                        &model,
                        &state_ref,
                        &effects_ref,
                        &mut pending_action,
                        &mut passive_mode,
                        config.confidence_threshold,
                        RequestKind::Prompt(prompt),
                    );
                }
                Ok(RuntimeCommand::Decision(accept)) => {
                    if let Some(action) = pending_action.take() {
                        if accept {
                            push_log(
                                &state_ref,
                                LogEntry {
                                    role: LogRole::System,
                                    text: "action confirmed".to_string(),
                                    timestamp_ms: now_ms(),
                                },
                            );
                            apply_action(&state_ref, &effects_ref, &action);
                        } else {
                            push_log(
                                &state_ref,
                                LogEntry {
                                    role: LogRole::System,
                                    text: "action rejected".to_string(),
                                    timestamp_ms: now_ms(),
                                },
                            );
                            set_ui(
                                &state_ref,
                                UiUpdate {
                                    status: "Watching".to_string(),
                                    message: Some("action rejected".to_string()),
                                    pending_action: None,
                                    dry_run: None,
                                },
                            );
                        }
                    }
                }
                Ok(RuntimeCommand::SetPassive(passive)) => passive_mode = passive,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }

            // Collect observer events into the buffer.
            while let Ok(event) = event_rx.try_recv() {
                event_buffer.push(event);
            }

            // Only process buffered events if the throttle has expired and
            // there's no pending action waiting for user input.
            if !event_buffer.is_empty()
                && pending_action.is_none()
                && last_event_call.elapsed() >= Duration::from_secs(EVENT_THROTTLE_SECS)
            {
                // Take the most important event (first one) and log all.
                let events = std::mem::take(&mut event_buffer);
                let labels: Vec<String> =
                    events.iter().map(event_label).collect();
                let combined_intent = format!(
                    "Respond to workspace events: {}",
                    labels.join(", ")
                );

                for label in &labels {
                    push_log(
                        &state_ref,
                        LogEntry {
                            role: LogRole::System,
                            text: format!("observed {label}"),
                            timestamp_ms: now_ms(),
                        },
                    );
                }

                last_event_call = Instant::now();
                process_request(
                    &model,
                    &state_ref,
                    &effects_ref,
                    &mut pending_action,
                    &mut passive_mode,
                    config.confidence_threshold,
                    RequestKind::BatchedEvents(combined_intent, events),
                );
            }
        }
    });

    AgentRuntimeHandle {
        tx,
        state,
        ui_effects,
    }
}

enum RequestKind {
    BatchedEvents(String, Vec<WorkspaceEvent>),
    Prompt(String),
}

fn process_request(
    model: &Arc<dyn AgentModel>,
    state: &Arc<Mutex<RuntimeState>>,
    effects: &Arc<Mutex<VecDeque<UiEffect>>>,
    pending_action: &mut Option<AgentAction>,
    passive_mode: &mut bool,
    confidence_threshold: f32,
    request_kind: RequestKind,
) {
    let intent = match &request_kind {
        RequestKind::BatchedEvents(intent, _) => intent.clone(),
        RequestKind::Prompt(prompt) => prompt.clone(),
    };

    set_ui(
        state,
        UiUpdate {
            status: "Thinking".to_string(),
            message: Some(match &request_kind {
                RequestKind::BatchedEvents(_, _) => "processing events".to_string(),
                RequestKind::Prompt(_) => "processing prompt".to_string(),
            }),
            pending_action: None,
            dry_run: None,
        },
    );

    let request = ModelRequest {
        intent,
        context: build_workspace_context(),
    };
    let actions = model.plan(&request).unwrap_or_else(|err| {
        push_log(
            state,
            LogEntry {
                role: LogRole::System,
                text: format!("model error: {err}"),
                timestamp_ms: now_ms(),
            },
        );
        match &request_kind {
            RequestKind::BatchedEvents(_, events) => {
                // Use fallback for the first event only.
                events.first().map(fallback_actions).unwrap_or_default()
            }
            RequestKind::Prompt(prompt) => vec![AgentAction::SurfaceMessage {
                message: format!("No model output for prompt: {prompt}"),
                confidence: 0.25,
            }],
        }
    });

    if actions.is_empty() {
        push_log(
            state,
            LogEntry {
                role: LogRole::Agent,
                text: "no action needed".to_string(),
                timestamp_ms: now_ms(),
            },
        );
        set_ui(
            state,
            UiUpdate {
                status: "Watching".to_string(),
                message: Some("no action".to_string()),
                pending_action: None,
                dry_run: None,
            },
        );
        return;
    }

    for action in &actions {
        let dry_run = action.dry_run();
        log_action(&request_kind, action, &dry_run);

        let needs_confirm = if *passive_mode {
            // Passive mode: confirm everything.
            true
        } else if action.is_non_destructive() && action.confidence() >= confidence_threshold {
            // Non-destructive + high confidence: auto-execute.
            false
        } else if matches!(action, AgentAction::RunCommand { .. }) {
            // Commands always need confirmation.
            true
        } else {
            // Low confidence or destructive: confirm.
            action.confidence() < confidence_threshold || !action.is_non_destructive()
        };

        if needs_confirm {
            push_log(
                state,
                LogEntry {
                    role: LogRole::Agent,
                    text: format!(
                        "proposed: {dry_run} (confidence {:.0}%)",
                        action.confidence() * 100.0
                    ),
                    timestamp_ms: now_ms(),
                },
            );
            *pending_action = Some(action.clone());
            set_ui(
                state,
                UiUpdate {
                    status: "Awaiting".to_string(),
                    message: Some("action proposed".to_string()),
                    pending_action: Some(action.clone()),
                    dry_run: Some(dry_run),
                },
            );
            // Stop processing further actions until this one is resolved.
            return;
        }

        push_log(
            state,
            LogEntry {
                role: LogRole::Agent,
                text: format!("executing: {dry_run}"),
                timestamp_ms: now_ms(),
            },
        );
        apply_action(state, effects, action);
    }
}

fn apply_action(
    state: &Arc<Mutex<RuntimeState>>,
    effects: &Arc<Mutex<VecDeque<UiEffect>>>,
    action: &AgentAction,
) {
    match execute_side_effect(action) {
        Ok(effect) => {
            if let Some(effect) = effect {
                if let Ok(mut queue) = effects.lock() {
                    queue.push_back(effect.clone());
                }
                let message = match &effect {
                    UiEffect::OpenPane(pane) => format!("opened {pane:?}"),
                    UiEffect::Message(message) => message.clone(),
                };
                push_log(
                    state,
                    LogEntry {
                        role: LogRole::Action,
                        text: message.clone(),
                        timestamp_ms: now_ms(),
                    },
                );
                set_ui(
                    state,
                    UiUpdate {
                        status: "Watching".to_string(),
                        message: Some(message),
                        pending_action: None,
                        dry_run: None,
                    },
                );
            }
        }
        Err(error) => {
            push_log(
                state,
                LogEntry {
                    role: LogRole::System,
                    text: format!("error: {error}"),
                    timestamp_ms: now_ms(),
                },
            );
            set_ui(
                state,
                UiUpdate {
                    status: "Watching".to_string(),
                    message: Some(error),
                    pending_action: None,
                    dry_run: None,
                },
            );
        }
    }
}

fn fallback_actions(event: &WorkspaceEvent) -> Vec<AgentAction> {
    match event {
        WorkspaceEvent::MergeConflict { files } => vec![
            AgentAction::OpenPane {
                pane: super::actions::PaneType::Git,
                confidence: 0.97,
            },
            AgentAction::SurfaceMessage {
                message: format!("Merge conflict detected in {}", files.join(", ")),
                confidence: 0.99,
            },
        ],
        WorkspaceEvent::BranchDiverged { ahead, behind } => vec![AgentAction::SurfaceMessage {
            message: format!("Branch diverged from remote (+{ahead}/-{behind})"),
            confidence: 0.94,
        }],
        WorkspaceEvent::CommandFailed {
            command,
            exit_code,
            ..
        } => vec![AgentAction::SurfaceMessage {
            message: format!("Last command failed with {exit_code}: {command}"),
            confidence: 0.91,
        }],
        WorkspaceEvent::RiskyHunksUnmarked { count } => vec![
            AgentAction::OpenPane {
                pane: super::actions::PaneType::Git,
                confidence: 0.93,
            },
            AgentAction::SurfaceMessage {
                message: format!("{count} risky hunks still need annotation"),
                confidence: 0.89,
            },
        ],
        _ => Vec::new(),
    }
}

fn push_log(state: &Arc<Mutex<RuntimeState>>, entry: LogEntry) {
    if let Ok(mut guard) = state.lock() {
        guard.log.push_back(entry);
        while guard.log.len() > MAX_LOG_ENTRIES {
            guard.log.pop_front();
        }
    }
}

fn set_ui(state: &Arc<Mutex<RuntimeState>>, next: UiUpdate) {
    if let Ok(mut guard) = state.lock() {
        guard.ui = next;
    }
}

fn log_action(request_kind: &RequestKind, action: &AgentAction, dry_run: &str) {
    let path = data_root().join("agent.log");
    ensure_parent(&path);
    let line = serde_json::json!({
        "ts": now_ms(),
        "source": match request_kind {
            RequestKind::BatchedEvents(_, _) => "event".to_string(),
            RequestKind::Prompt(_) => "prompt".to_string(),
        },
        "action": action,
        "dry_run": dry_run,
        "context": token_budgeted_json(&build_workspace_context(), 1024),
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}

fn event_label(event: &WorkspaceEvent) -> String {
    match event {
        WorkspaceEvent::ErrorRateSpike { .. } => "error_rate_spike",
        WorkspaceEvent::RepeatedError { .. } => "repeated_error",
        WorkspaceEvent::NewErrorPattern { .. } => "new_error_pattern",
        WorkspaceEvent::CommandFailed { .. } => "command_failed",
        WorkspaceEvent::CommandPreviouslyFailed { .. } => "command_previously_failed",
        WorkspaceEvent::MergeConflict { .. } => "merge_conflict",
        WorkspaceEvent::BranchDiverged { .. } => "branch_diverged",
        WorkspaceEvent::UncommittedIdle { .. } => "uncommitted_idle",
        WorkspaceEvent::RiskyHunksUnmarked { .. } => "risky_hunks_unmarked",
    }
    .to_string()
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
