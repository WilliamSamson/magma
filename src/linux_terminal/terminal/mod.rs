#![allow(unused_imports)]
pub(crate) mod mux;
pub(crate) mod profile;
pub(crate) mod runtime;
pub(crate) mod session;
pub(crate) mod shell;
pub(crate) mod tab;
pub(crate) mod widget;

pub(crate) use mux::MuxPaneView;
pub(crate) use profile::{ProfileId, TerminalProfile, next_profile, profile};
pub(crate) use runtime::resolve_shell;
pub(crate) use session::SessionView;
pub(crate) use shell::{ShellRuntime, spawn_shell};
pub(crate) use tab::TabView;
pub(crate) use widget::{apply_terminal_settings, build_terminal, scaled_spacing};
