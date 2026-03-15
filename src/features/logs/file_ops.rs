use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

/// Removes a specific line from a file given its 1-indexed line number.
/// This is a destructive operation that overwrites the file.
pub(crate) fn remove_line_at(path: &Path, line_num: usize) -> io::Result<()> {
    if line_num == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Line number must be 1-indexed"));
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    // Collect all lines except the one to be removed
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index + 1 != line_num {
            lines.push(line);
        }
    }

    // Write back to the file
    let mut file = File::create(path)?;
    for line in lines {
        writeln!(file, "{line}")?;
    }

    Ok(())
}
