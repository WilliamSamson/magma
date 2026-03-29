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
    pub(super) category: FileCategory,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum FileCategory {
    #[default]
    All,
    Rust,
    Dart,
    Python,
    JsTs,
    Java,
    Go,
    Cpp,
    Swift,
    Web,
    Shell,
    Config,
    Docs,
    Images,
    Other,
}

pub(super) fn scan_directory(dir: &Path) -> Vec<ViewerFile> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let kind = kind_for_path(&path)?;
            let category = category_for_path(&path, kind);
            let metadata = entry.metadata().ok()?;
            Some(ViewerFile {
                name: path.file_name()?.to_string_lossy().to_string(),
                path,
                kind,
                category,
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

/// Construct a ViewerFile from an arbitrary path, if the view pane supports it.
pub(super) fn viewer_file_for_path(path: &Path) -> Option<ViewerFile> {
    let kind = kind_for_path(path)?;
    let category = category_for_path(path, kind);
    let metadata = fs::metadata(path).ok()?;
    Some(ViewerFile {
        name: path.file_name()?.to_string_lossy().to_string(),
        path: path.to_path_buf(),
        kind,
        category,
        size_bytes: metadata.len(),
        modified_secs: metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| duration.as_secs()),
    })
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

fn category_for_path(path: &Path, kind: FileKind) -> FileCategory {
    match kind {
        FileKind::Image => return FileCategory::Images,
        FileKind::Pdf | FileKind::Docx | FileKind::Office => return FileCategory::Docs,
        FileKind::Code => {}
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("rs" | "rt") => FileCategory::Rust,
        Some("dart") => FileCategory::Dart,
        Some("py" | "pyw" | "r") => FileCategory::Python,
        Some("js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx") => FileCategory::JsTs,
        Some("java" | "kt" | "kts" | "scala" | "groovy" | "gradle") => FileCategory::Java,
        Some("go") => FileCategory::Go,
        Some("c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" | "cs") => FileCategory::Cpp,
        Some("swift") => FileCategory::Swift,
        Some("html" | "htm" | "css" | "scss" | "sass" | "less" | "xml" | "vue" | "svelte" | "astro") => FileCategory::Web,
        Some("sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "bat" | "cmd") => FileCategory::Shell,
        Some("json" | "jsonc" | "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" | "env"
            | "nix" | "sql" | "tf" | "hcl" | "proto" | "graphql" | "gql" | "prisma" | "cmake") => FileCategory::Config,
        Some("md" | "txt" | "log" | "rst") => FileCategory::Docs,
        _ => FileCategory::Other,
    }
}

pub(super) fn category_label(cat: FileCategory) -> &'static str {
    match cat {
        FileCategory::All => "all",
        FileCategory::Rust => "rust",
        FileCategory::Dart => "dart",
        FileCategory::Python => "python",
        FileCategory::JsTs => "js/ts",
        FileCategory::Java => "java",
        FileCategory::Go => "go",
        FileCategory::Cpp => "c/c++",
        FileCategory::Swift => "swift",
        FileCategory::Web => "web",
        FileCategory::Shell => "shell",
        FileCategory::Config => "config",
        FileCategory::Docs => "docs",
        FileCategory::Images => "images",
        FileCategory::Other => "other",
    }
}

pub(super) fn category_order(cat: FileCategory) -> u8 {
    match cat {
        FileCategory::All => 0,
        FileCategory::Rust => 1,
        FileCategory::Dart => 2,
        FileCategory::Python => 3,
        FileCategory::JsTs => 4,
        FileCategory::Java => 5,
        FileCategory::Go => 6,
        FileCategory::Cpp => 7,
        FileCategory::Swift => 8,
        FileCategory::Web => 9,
        FileCategory::Shell => 10,
        FileCategory::Config => 11,
        FileCategory::Docs => 12,
        FileCategory::Images => 13,
        FileCategory::Other => 14,
    }
}
