use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use super::context::build_workspace_context;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum WorkspaceEvent {
    ErrorRateSpike { count: usize },
    RepeatedError { message: String, count: usize },
    NewErrorPattern { message: String },
    CommandFailed { branch: String, command: String, exit_code: i32 },
    CommandPreviouslyFailed { branch: String, command: String },
    MergeConflict { files: Vec<String> },
    BranchDiverged { ahead: u32, behind: u32 },
    UncommittedIdle { minutes: u64, changed_files: usize },
    RiskyHunksUnmarked { count: usize },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ObserverConfig {
    pub(crate) poll_ms: u64,
    pub(crate) idle_after_secs: u64,
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self {
            poll_ms: 3_000,
            idle_after_secs: 180,
        }
    }
}

/// Minimum seconds between re-emitting the same event kind.
const COOLDOWN_SECS: u64 = 120;

/// Seconds to wait after startup before emitting any events — lets the
/// workspace settle and avoids burning API quota on stale state.
const STARTUP_DELAY_SECS: u64 = 15;

pub(crate) fn spawn(config: ObserverConfig) -> Receiver<WorkspaceEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // Wait for the workspace to settle before polling.
        thread::sleep(Duration::from_secs(STARTUP_DELAY_SECS));

        let mut seen_errors: HashSet<String> = HashSet::new();
        let mut failed_by_branch: HashMap<String, HashSet<String>> = HashMap::new();
        let mut dirty_since: Option<Instant> = None;
        let mut last_emitted: HashMap<&str, Instant> = HashMap::new();

        loop {
            let snapshot = build_workspace_context();
            let mut candidates: Vec<(&str, WorkspaceEvent)> = Vec::new();

            // ── Error analysis ──
            let errors: Vec<String> = snapshot
                .logs
                .last_entries
                .iter()
                .filter(|entry| entry.level == "ERROR" || entry.level == "WARN")
                .map(|entry| entry.message.clone())
                .collect();

            if errors.len() >= 8 {
                candidates.push((
                    "error_rate_spike",
                    WorkspaceEvent::ErrorRateSpike { count: errors.len() },
                ));
            }
            if let Some((message, count)) = repeated_error(&errors) {
                candidates.push((
                    "repeated_error",
                    WorkspaceEvent::RepeatedError { message, count },
                ));
            }
            for message in errors.iter().take(3).cloned() {
                if seen_errors.insert(message.clone()) {
                    candidates.push((
                        "new_error_pattern",
                        WorkspaceEvent::NewErrorPattern { message },
                    ));
                }
            }

            // ── Command failures ──
            if let (Some(branch), Some(code), Some(command)) = (
                snapshot.git.branch.clone(),
                snapshot.terminal.last_exit_code,
                snapshot.terminal.last_command.clone(),
            ) {
                if code != 0 {
                    let seen = failed_by_branch.entry(branch.clone()).or_default();
                    if seen.contains(&command) {
                        candidates.push((
                            "command_previously_failed",
                            WorkspaceEvent::CommandPreviouslyFailed {
                                branch: branch.clone(),
                                command: command.clone(),
                            },
                        ));
                    } else {
                        candidates.push((
                            "command_failed",
                            WorkspaceEvent::CommandFailed {
                                branch: branch.clone(),
                                command: command.clone(),
                                exit_code: code,
                            },
                        ));
                        seen.insert(command);
                    }
                }
            }

            // ── Git state ──
            if !snapshot.git.conflicted.is_empty() {
                candidates.push((
                    "merge_conflict",
                    WorkspaceEvent::MergeConflict {
                        files: snapshot.git.conflicted.clone(),
                    },
                ));
            }
            if snapshot.git.ahead > 0 && snapshot.git.behind > 0 {
                candidates.push((
                    "branch_diverged",
                    WorkspaceEvent::BranchDiverged {
                        ahead: snapshot.git.ahead,
                        behind: snapshot.git.behind,
                    },
                ));
            }

            // ── Uncommitted idle ──
            let dirty_count =
                snapshot.git.staged_summary.len() + snapshot.git.unstaged_summary.len();
            dirty_since = match (dirty_count > 0, dirty_since) {
                (true, None) => Some(Instant::now()),
                (false, _) => None,
                (true, since) => since,
            };
            if let Some(since) = dirty_since {
                let minutes = since.elapsed().as_secs() / 60;
                if since.elapsed().as_secs() >= config.idle_after_secs {
                    candidates.push((
                        "uncommitted_idle",
                        WorkspaceEvent::UncommittedIdle {
                            minutes,
                            changed_files: dirty_count,
                        },
                    ));
                }
            }

            // ── Risky hunks ──
            let risky_unannotated = snapshot
                .annotations
                .iter()
                .filter(|item| item.status == "Risky" && item.annotation.trim().is_empty())
                .count();
            if risky_unannotated > 0 {
                candidates.push((
                    "risky_hunks_unmarked",
                    WorkspaceEvent::RiskyHunksUnmarked {
                        count: risky_unannotated,
                    },
                ));
            }

            // ── Cooldown filter: only emit if enough time has passed ──
            let now = Instant::now();
            for (kind, event) in candidates {
                let cooled = last_emitted
                    .get(kind)
                    .map(|last| now.duration_since(*last) >= Duration::from_secs(COOLDOWN_SECS))
                    .unwrap_or(true);
                if cooled {
                    last_emitted.insert(kind, now);
                    if tx.send(event).is_err() {
                        return;
                    }
                }
            }

            thread::sleep(Duration::from_millis(config.poll_ms));
        }
    });
    rx
}

fn repeated_error(errors: &[String]) -> Option<(String, usize)> {
    let mut counts = HashMap::new();
    for message in errors {
        *counts.entry(message.clone()).or_insert(0usize) += 1;
    }
    counts.into_iter().find(|(_, count)| *count >= 3)
}
