use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use gtk::{gio, glib};
use vte4::{prelude::*, PtyFlags, Terminal};

use super::{persist::SessionSnapshot, runtime};

#[allow(dead_code)]
pub(super) struct ShellRuntime {
    status_path: PathBuf,
}

#[allow(dead_code)]
impl ShellRuntime {
    pub(super) fn status_path(&self) -> &Path {
        &self.status_path
    }
}

pub(super) fn spawn_shell(
    terminal: &Terminal,
    session: &SessionSnapshot,
    shell_override: &str,
) -> ShellRuntime {
    let shell = runtime::resolve_shell(shell_override);
    let status_path = session
        .status_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(status_path);
    ensure_status_parent(&status_path);

    let home = std::env::var("HOME").ok();
    let cwd = session.cwd.as_deref().or(home.as_deref());

    let launch = build_launch(&shell, session, &status_path, cwd);

    let args_refs: Vec<&str> = launch.args.iter().map(String::as_str).collect();
    let env_refs: Vec<&str> = launch.env.iter().map(String::as_str).collect();

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

struct ShellLaunch {
    args: Vec<String>,
    env: Vec<String>,
}

fn build_launch(
    shell: &str,
    session: &SessionSnapshot,
    status_path: &Path,
    cwd: Option<&str>,
) -> ShellLaunch {
    let env = shell_env(status_path);

    if let Some(tmux) = runtime::resolve_executable("tmux") {
        if let (Some(session_id), Some(socket_path)) =
            (session.session_id.as_deref(), session.socket_path.as_deref())
        {
            return ShellLaunch {
                args: tmux_args(&tmux, shell, status_path, session_id, socket_path, cwd),
                env,
            };
        }
    }

    ShellLaunch {
        args: shell_args(shell),
        env,
    }
}

fn shell_env(status_path: &Path) -> Vec<String> {
    let mut env = std::env::vars_os()
        .map(|(key, value)| format!("{}={}", key.to_string_lossy(), value.to_string_lossy()))
        .collect::<Vec<_>>();
    set_env(&mut env, "TERM", "xterm-256color");
    set_env(&mut env, "COLORTERM", "truecolor");
    set_env(&mut env, "TERM_PROGRAM", "magma");
    ensure_utf8_locale(&mut env);
    set_env(
        &mut env,
        "MAGMA_STATUS_FILE",
        &status_path.to_string_lossy(),
    );
    env
}

fn tmux_args(
    tmux: &Path,
    shell: &str,
    status_path: &Path,
    session_id: &str,
    socket_path: &str,
    cwd: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        tmux.display().to_string(),
        "-S".to_string(),
        socket_path.to_string(),
        "new-session".to_string(),
        "-A".to_string(),
        "-D".to_string(),
        "-s".to_string(),
        session_id.to_string(),
    ];

    if let Some(cwd) = cwd {
        args.push("-c".to_string());
        args.push(cwd.to_string());
    }

    args.push(tmux_shell_command(shell, status_path));
    args
}

fn tmux_shell_command(shell: &str, status_path: &Path) -> String {
    let mut command = vec!["env".to_string()];
    for item in shell_env_items(status_path) {
        command.push(shell_quote(&format!("{}={}", item.0, item.1)));
    }
    for arg in shell_args(shell) {
        command.push(shell_quote(&arg));
    }
    command.join(" ")
}

fn ensure_utf8_locale(env: &mut Vec<String>) {
    let preferred = current_utf8_locale().unwrap_or_else(|| "C.UTF-8".to_string());
    for key in ["LANG", "LC_CTYPE", "LC_ALL"] {
        let existing = find_env(env, key);
        if existing.is_none_or(|value| !is_utf8_locale(value)) {
            set_env(env, key, &preferred);
        }
    }
}

fn current_utf8_locale() -> Option<String> {
    for key in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            if is_utf8_locale(&value) {
                return Some(value);
            }
        }
    }
    None
}

fn find_env<'a>(env: &'a [String], key: &str) -> Option<&'a str> {
    let needle = format!("{key}=");
    env.iter()
        .find(|item| item.starts_with(&needle))
        .and_then(|item| item.split_once('='))
        .map(|(_, value)| value)
}

fn is_utf8_locale(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("utf-8") || lowered.contains("utf8")
}

fn set_env(env: &mut Vec<String>, key: &str, value: &str) {
    let needle = format!("{key}=");
    if let Some(existing) = env.iter_mut().find(|item| item.starts_with(&needle)) {
        *existing = format!("{key}={value}");
        return;
    }

    env.push(format!("{key}={value}"));
}

fn shell_env_items(status_path: &Path) -> Vec<(String, String)> {
    let mut env = vec![
        ("TERM".to_string(), "xterm-256color".to_string()),
        ("COLORTERM".to_string(), "truecolor".to_string()),
        ("TERM_PROGRAM".to_string(), "magma".to_string()),
        (
            "MAGMA_STATUS_FILE".to_string(),
            status_path.to_string_lossy().to_string(),
        ),
    ];
    ensure_utf8_locale_items(&mut env);
    env
}

fn ensure_utf8_locale_items(env: &mut Vec<(String, String)>) {
    let preferred = current_utf8_locale().unwrap_or_else(|| "C.UTF-8".to_string());
    for key in ["LANG", "LC_CTYPE", "LC_ALL"] {
        let index = env.iter().position(|(existing, _)| existing == key);
        if let Some(index) = index {
            if !is_utf8_locale(&env[index].1) {
                env[index].1 = preferred.clone();
            }
            continue;
        }

        match std::env::var(key) {
            Ok(value) if is_utf8_locale(&value) => env.push((key.to_string(), value)),
            _ => env.push((key.to_string(), preferred.clone())),
        }
    }
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
__magma_status_update() {
    local exit_code=$?
    local command_text="${MAGMA_PENDING_COMMAND:-}"
    if [ "${MAGMA_STATUS_READY:-0}" != "1" ]; then
        MAGMA_STATUS_READY=1
        MAGMA_AT_PROMPT=1
        return
    fi
    if [ -n "${MAGMA_STATUS_FILE:-}" ]; then
        if [ -n "$command_text" ]; then
            MAGMA_STATUS_SEQ=$(( ${MAGMA_STATUS_SEQ:-0} + 1 ))
        fi
        printf "%s\t%s\t%s\n" "${MAGMA_STATUS_SEQ:-0}" "$exit_code" "$command_text" > "$MAGMA_STATUS_FILE"
    fi
    MAGMA_PENDING_COMMAND=""
    MAGMA_AT_PROMPT=1
}
MAGMA_STATUS_SEQ=0
MAGMA_STATUS_READY=0
MAGMA_AT_PROMPT=0
MAGMA_PENDING_COMMAND=""
__magma_capture_command() {
    if [ "${MAGMA_AT_PROMPT:-0}" != "1" ]; then
        return
    fi
    MAGMA_PENDING_COMMAND="$BASH_COMMAND"
    MAGMA_AT_PROMPT=0
}
trap '__magma_capture_command' DEBUG
if [[ "$(declare -p PROMPT_COMMAND 2>&1)" =~ "declare -a" ]]; then
    PROMPT_COMMAND+=(__magma_status_update)
elif [ -n "${PROMPT_COMMAND:-}" ]; then
    PROMPT_COMMAND="__magma_status_update; ${PROMPT_COMMAND}"
else
    PROMPT_COMMAND="__magma_status_update"
fi
clear
"#;

    let mut file = File::create(&path)?;
    file.write_all(rc_content.as_bytes())?;
    Ok(path.to_string_lossy().to_string())
}

fn rc_path() -> PathBuf {
    std::env::temp_dir().join(format!("magma_bashrc_{}", std::process::id()))
}

fn status_path() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "magma_shell_status_{}_{}",
        std::process::id(),
        timestamp_nanos()
    ));
    let _ = std::fs::write(&path, "0\t0\t\n");
    path
}

fn ensure_status_parent(status_path: &Path) {
    if let Some(parent) = status_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !status_path.exists() {
        let _ = std::fs::write(status_path, "0\t0\t\n");
    }
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    }
}
