use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use super::code::supports_code_preview;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ViewerFile {
    pub(super) path: PathBuf,
    pub(super) name: String,
    pub(super) kind: FileKind,
    pub(super) size_bytes: u64,
    modified_secs: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FileKind {
    Image,
    Pdf,
    Docx,
    Code,
    Office,
}

pub(super) fn scan_directory(dir: &Path) -> Vec<ViewerFile> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let kind = kind_for_path(&path)?;
            let metadata = entry.metadata().ok()?;
            Some(ViewerFile {
                name: path.file_name()?.to_string_lossy().to_string(),
                path,
                kind,
                size_bytes: metadata.len(),
                modified_secs: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |duration| duration.as_secs()),
            })
        })
        .collect();

    files.sort_by(|left, right| {
        right
            .modified_secs
            .cmp(&left.modified_secs)
            .then_with(|| left.name.cmp(&right.name))
    });
    files
}

pub(super) fn format_size(size_bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = size_bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        return format!("{size_bytes} {}", UNITS[unit]);
    }

    format!("{value:.1} {}", UNITS[unit])
}

pub(super) fn kind_label(kind: FileKind) -> &'static str {
    match kind {
        FileKind::Image => "image",
        FileKind::Pdf => "pdf",
        FileKind::Docx => "docx",
        FileKind::Code => "code",
        FileKind::Office => "document",
    }
}

fn kind_for_path(path: &Path) -> Option<FileKind> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match ext.as_deref() {
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg") => Some(FileKind::Image),
        Some("pdf") => Some(FileKind::Pdf),
        Some("docx") => Some(FileKind::Docx),
        Some("doc" | "ppt" | "pptx") => Some(FileKind::Office),
        _ if supports_code_preview(path) => Some(FileKind::Code),
        _ => None,
    }
}
