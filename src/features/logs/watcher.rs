use std::{
    fs::File,
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use super::{parser::FollowConfig, log_entry::LogEntry};

pub(crate) fn spawn_file_follower(config: FollowConfig) -> Receiver<LogEntry> {
    let (sender, receiver) = mpsc::channel();
    let _ = thread::Builder::new()
        .name("magma-logs-follower".to_string())
        .spawn(move || follow_file(config, sender));
    receiver
}

fn follow_file(mut config: FollowConfig, sender: Sender<LogEntry>) {
    let file = match File::open(&config.path) {
        Ok(file) => file,
        Err(error) => {
            send_follow_error(&sender, config.next_line_number, error);
            return;
        }
    };

    let mut reader = BufReader::new(file);
    if let Err(error) = reader.seek(SeekFrom::Start(config.offset)) {
        send_follow_error(&sender, config.next_line_number, error);
        return;
    }

    let mut pending = String::new();
    loop {
        match read_next_line(&mut reader, &mut pending) {
            Ok(Some(line)) => {
                if send_entry(&sender, config.next_line_number, &line).is_err() {
                    return;
                }
                config.next_line_number += 1;
            }
            Ok(None) => thread::sleep(Duration::from_millis(250)),
            Err(error) => {
                send_follow_error(&sender, config.next_line_number, error);
                return;
            }
        }
    }
}

fn read_next_line(reader: &mut BufReader<File>, pending: &mut String) -> io::Result<Option<String>> {
    let mut chunk = String::new();
    let bytes_read = reader.read_line(&mut chunk)?;

    if bytes_read == 0 {
        return Ok(None);
    }

    pending.push_str(&chunk);
    if !pending.ends_with('\n') {
        return Ok(None);
    }

    let line = pending.trim_end_matches(['\n', '\r']).to_string();
    pending.clear();
    Ok(Some(line))
}

fn send_entry(sender: &Sender<LogEntry>, line_number: usize, line: &str) -> io::Result<()> {
    let entry = LogEntry::from_json_line(line_number, line);
    sender
        .send(entry)
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "log viewer closed"))
}

fn send_follow_error(sender: &Sender<LogEntry>, line_number: usize, error: io::Error) {
    let message = format!("follow error: {error}");
    let _ = sender.send(LogEntry::follow_error(line_number, message));
}
