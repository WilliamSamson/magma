use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PaneType {
    Logr,
    Web,
    View,
    Git,
    Agent,
    Notes,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct LogFilter {
    pub(crate) query: Option<String>,
    pub(crate) levels: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct HunkRef {
    pub(crate) file: String,
    pub(crate) hunk_index: usize,
    pub(crate) branch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum AgentAction {
    RunCommand { command: String, confidence: f32 },
    OpenPane { pane: PaneType, confidence: f32 },
    FilterLogr { filter: LogFilter, confidence: f32 },
    StageHunk { hunk: HunkRef, confidence: f32 },
    WriteAnnotation { hunk: HunkRef, note: String, confidence: f32 },
    SurfaceMessage { message: String, confidence: f32 },
}

impl AgentAction {
    pub(crate) fn confidence(&self) -> f32 {
        match self {
            Self::RunCommand { confidence, .. }
            | Self::OpenPane { confidence, .. }
            | Self::FilterLogr { confidence, .. }
            | Self::StageHunk { confidence, .. }
            | Self::WriteAnnotation { confidence, .. }
            | Self::SurfaceMessage { confidence, .. } => *confidence,
        }
    }

    pub(crate) fn dry_run(&self) -> String {
        match self {
            Self::RunCommand { command, .. } => format!("Would run `{command}` in the current workspace"),
            Self::OpenPane { pane, .. } => format!("Would open the `{}` side pane", pane.label()),
            Self::FilterLogr { filter, .. } => format!(
                "Would filter logr to query={:?} levels={}",
                filter.query,
                if filter.levels.is_empty() { "all".to_string() } else { filter.levels.join(",") }
            ),
            Self::StageHunk { hunk, .. } => {
                format!("Would stage hunk {} from {}", hunk.hunk_index, hunk.file)
            }
            Self::WriteAnnotation { hunk, note, .. } => format!(
                "Would write annotation on {}#{}: {}",
                hunk.file, hunk.hunk_index, note
            ),
            Self::SurfaceMessage { message, .. } => format!("Would surface message: {message}"),
        }
    }

    pub(crate) fn is_non_destructive(&self) -> bool {
        matches!(
            self,
            Self::OpenPane { .. } | Self::FilterLogr { .. } | Self::SurfaceMessage { .. }
        )
    }
}

impl PaneType {
    fn label(&self) -> &'static str {
        match self {
            Self::Logr => "logr",
            Self::Web => "web",
            Self::View => "viewer",
            Self::Git => "git",
            Self::Agent => "agent",
            Self::Notes => "notes",
        }
    }
}
