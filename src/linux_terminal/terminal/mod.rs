#![allow(unused_imports)]
pub(crate) mod mux;
pub(crate) mod profile;
pub(crate) mod runtime;
pub(crate) mod session;
pub(crate) mod shell;
pub(crate) mod tab;
pub(crate) mod widget;

pub(crate) use mux::MuxPaneView;
pub(crate) use profile::{next_profile, profile, ProfileId, TerminalProfile};
pub(crate) use runtime::resolve_shell;
pub(crate) use session::SessionView;
pub(crate) use shell::{spawn_shell, ShellRuntime};
pub(crate) use tab::TabView;
pub(crate) use widget::{apply_terminal_settings, build_terminal, scaled_spacing};
