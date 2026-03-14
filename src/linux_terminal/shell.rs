use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use gtk::{gio, glib};
use vte4::{prelude::*, PtyFlags, Terminal};

pub(super) struct ShellRuntime {
    status_path: PathBuf,
}

impl ShellRuntime {
    pub(super) fn status_path(&self) -> &Path {
        &self.status_path
    }
}

pub(super) fn spawn_shell(terminal: &Terminal, working_directory: Option<&str>, shell_override: &str) -> ShellRuntime {
    let shell = if shell_override.is_empty() {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    } else {
        shell_override.to_string()
    };
    let status_path = status_path();
    let args = shell_args(&shell);
    let env = shell_env(&status_path);

    let home = std::env::var("HOME").ok();
    let cwd = working_directory.or(home.as_deref());

    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let env_refs: Vec<&str> = env.iter().map(String::as_str).collect();

    terminal.spawn_async(
        PtyFlags::DEFAULT,
        cwd,
        &args_refs,
        &env_refs,
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        move |result| {
            if let Err(error) = result {
                eprintln!("terminal spawn failed: {error}");
            }
        },
    );

    ShellRuntime { status_path }
}

fn shell_env(status_path: &Path) -> Vec<String> {
    let mut env = std::env::vars_os()
        .map(|(key, value)| format!("{}={}", key.to_string_lossy(), value.to_string_lossy()))
        .collect::<Vec<_>>();
    set_env(&mut env, "TERM", "xterm-256color");
    set_env(&mut env, "COLORTERM", "truecolor");
    set_env(&mut env, "TERM_PROGRAM", "obsidian");
    set_env(
        &mut env,
        "OBSIDIAN_STATUS_FILE",
        &status_path.to_string_lossy(),
    );
    env
}

fn set_env(env: &mut Vec<String>, key: &str, value: &str) {
    let needle = format!("{key}=");
    if let Some(existing) = env.iter_mut().find(|item| item.starts_with(&needle)) {
        *existing = format!("{key}={value}");
        return;
    }

    env.push(format!("{key}={value}"));
}

fn shell_args(shell: &str) -> Vec<String> {
    if shell.ends_with("bash") {
        return bash_args(shell);
    }

    vec![shell.to_string(), "-i".to_string()]
}

fn bash_args(shell: &str) -> Vec<String> {
    match temp_rc_file() {
        // `-i` is required so builtins like `cd` run inside one persistent interactive shell.
        Ok(rc_path) => vec![
            shell.to_string(),
            "--noprofile".to_string(),
            "--rcfile".to_string(),
            rc_path,
            "-i".to_string(),
        ],
        Err(error) => {
            eprintln!("temporary rc file setup failed: {error}");
            vec![shell.to_string(), "-i".to_string()]
        }
    }
}

fn temp_rc_file() -> io::Result<String> {
    let path = rc_path();
    let rc_content = r#"
if [ -f ~/.bashrc ]; then
    source ~/.bashrc
fi
export PS1=""
if [ -s /etc/profile.d/vte-2.91.sh ]; then
    source /etc/profile.d/vte-2.91.sh
fi
__obsidian_status_update() {
    local exit_code=$?
    local command_text="${OBSIDIAN_PENDING_COMMAND:-}"
    if [ "${OBSIDIAN_STATUS_READY:-0}" != "1" ]; then
        OBSIDIAN_STATUS_READY=1
        OBSIDIAN_AT_PROMPT=1
        return
    fi
    if [ -n "${OBSIDIAN_STATUS_FILE:-}" ]; then
        if [ -n "$command_text" ]; then
            OBSIDIAN_STATUS_SEQ=$(( ${OBSIDIAN_STATUS_SEQ:-0} + 1 ))
        fi
        printf "%s\t%s\t%s\n" "${OBSIDIAN_STATUS_SEQ:-0}" "$exit_code" "$command_text" > "$OBSIDIAN_STATUS_FILE"
    fi
    OBSIDIAN_PENDING_COMMAND=""
    OBSIDIAN_AT_PROMPT=1
}
OBSIDIAN_STATUS_SEQ=0
OBSIDIAN_STATUS_READY=0
OBSIDIAN_AT_PROMPT=0
OBSIDIAN_PENDING_COMMAND=""
__obsidian_capture_command() {
    if [ "${OBSIDIAN_AT_PROMPT:-0}" != "1" ]; then
        return
    fi
    OBSIDIAN_PENDING_COMMAND="$BASH_COMMAND"
    OBSIDIAN_AT_PROMPT=0
}
trap '__obsidian_capture_command' DEBUG
if [[ "$(declare -p PROMPT_COMMAND 2>&1)" =~ "declare -a" ]]; then
    PROMPT_COMMAND+=(__obsidian_status_update)
elif [ -n "${PROMPT_COMMAND:-}" ]; then
    PROMPT_COMMAND="__obsidian_status_update; ${PROMPT_COMMAND}"
else
    PROMPT_COMMAND="__obsidian_status_update"
fi
clear
"#;

    let mut file = File::create(&path)?;
    file.write_all(rc_content.as_bytes())?;
    Ok(path.to_string_lossy().to_string())
}

fn rc_path() -> PathBuf {
    std::env::temp_dir().join("obsidian_bashrc")
}

fn status_path() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "obsidian_shell_status_{}_{}",
        std::process::id(),
        timestamp_nanos()
    ));
    let _ = std::fs::write(&path, "0\t0\t\n");
    path
}

fn timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    }
}
