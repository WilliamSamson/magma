use std::{fs, fs::File, io::{self, BufRead, BufReader, IsTerminal}};

use super::log_entry::LogEntry;

pub(crate) struct LoadedSource {
    pub(crate) source_name: String,
    pub(crate) entries: Vec<LogEntry>,
    pub(crate) follow_config: Option<FollowConfig>,
}

pub(crate) struct FollowConfig {
    pub(crate) path: String,
    pub(crate) offset: u64,
    pub(crate) next_line_number: usize,
}

pub(crate) fn load_source(input_path: Option<String>) -> io::Result<LoadedSource> {
    if let Some(path) = input_path {
        return load_file_source(path);
    }

    if io::stdin().is_terminal() {
        return Ok(LoadedSource {
            source_name: "no input".to_string(),
            entries: Vec::new(),
            follow_config: None,
        });
    }

    let stdin = io::stdin();
    Ok(LoadedSource {
        source_name: "stdin".to_string(),
        entries: read_entries(stdin.lock())?,
        follow_config: None,
    })
}

fn load_file_source(path: String) -> io::Result<LoadedSource> {
    let reader = BufReader::new(File::open(&path)?);
    let entries = read_entries(reader)?;
    let next_line_number = entries.last().map(|entry| entry.line_number() + 1).unwrap_or(1);
    let offset = fs::metadata(&path)?.len();

    Ok(LoadedSource {
        source_name: format!("file: {path}"),
        entries,
        follow_config: Some(FollowConfig {
            path,
            offset,
            next_line_number,
        }),
    })
}

fn read_entries<R: BufRead>(reader: R) -> io::Result<Vec<LogEntry>> {
    let mut entries = Vec::new();

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }

        entries.push(LogEntry::from_json_line(index + 1, &line));
    }

    Ok(entries)
}
