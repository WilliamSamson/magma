use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use super::log_entry::LogEntry;

pub(crate) fn write_filtered(path: &str, entries: &[&LogEntry]) -> io::Result<usize> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for entry in entries {
        writeln!(writer, "{}", entry.raw_line())?;
    }

    writer.flush()?;
    Ok(entries.len())
}
