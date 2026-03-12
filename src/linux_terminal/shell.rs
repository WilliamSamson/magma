use std::{fs::File, io::{self, Write}, path::PathBuf};

use gtk::{gio, glib};
use vte4::{prelude::*, PtyFlags, Terminal};

pub(super) fn spawn_shell(terminal: &Terminal) {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let args = shell_args(&shell);

    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    terminal.spawn_async(
        PtyFlags::DEFAULT,
        None,
        &args_refs,
        &[],
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
export PROMPT_COMMAND=""
clear
"#;

    let mut file = File::create(&path)?;
    file.write_all(rc_content.as_bytes())?;
    Ok(path.to_string_lossy().to_string())
}

fn rc_path() -> PathBuf {
    std::env::temp_dir().join("obsidian_bashrc")
}
