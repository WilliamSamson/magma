use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProfileId {
    Default,
    Focus,
    Compact,
}

pub(crate) struct TerminalProfile {
    pub(crate) label: &'static str,
    pub(crate) font_scale: f64,
}

pub(crate) fn profile(id: ProfileId) -> TerminalProfile {
    match id {
        ProfileId::Default => TerminalProfile {
            label: "Default",
            font_scale: 1.0,
        },
        ProfileId::Focus => TerminalProfile {
            label: "Focus",
            font_scale: 1.1,
        },
        ProfileId::Compact => TerminalProfile {
            label: "Compact",
            font_scale: 0.92,
        },
    }
}

pub(crate) fn next_profile(id: ProfileId) -> ProfileId {
    match id {
        ProfileId::Default => ProfileId::Focus,
        ProfileId::Focus => ProfileId::Compact,
        ProfileId::Compact => ProfileId::Default,
    }
}
