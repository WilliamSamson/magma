use std::path::Path;

pub(super) fn supports_code_preview(path: &Path) -> bool {
    extension(path)
        .is_some_and(|ext| CODE_EXTENSIONS.contains(&ext.as_str()))
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(is_known_text_filename)
}

pub(super) fn language_for_path(path: &Path) -> CodeLanguage {
    if let Some(ext) = extension(path) {
        if let Some(language) = language_for_extension(&ext) {
            return language;
        }
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(language_for_filename)
        .unwrap_or(CodeLanguage::Plain)
}

pub(super) fn language_for_text(text: &str) -> CodeLanguage {
    if text.contains("<!--") {
        return CodeLanguage::Markup;
    }
    if text.contains("/*") {
        return CodeLanguage::Css;
    }
    if text.contains("//") {
        return CodeLanguage::Slash;
    }
    if text.contains("--") {
        return CodeLanguage::Sql;
    }
    if text.contains('#') {
        return CodeLanguage::Hash;
    }
    CodeLanguage::Plain
}

pub(super) fn comment_tokens(language: CodeLanguage) -> &'static [&'static str] {
    match language {
        CodeLanguage::Hash => &["#"],
        CodeLanguage::Slash => &["//", "/*"],
        CodeLanguage::Sql => &["--"],
        CodeLanguage::Markup => &["<!--"],
        CodeLanguage::Css => &["/*"],
        CodeLanguage::Plain => &[],
    }
}

pub(super) fn keywords(language: CodeLanguage) -> &'static [&'static str] {
    match language {
        CodeLanguage::Hash => &[
            "alias", "and", "as", "async", "await", "case", "class", "def", "elif", "else",
            "esac", "except", "export", "false", "fi", "fn", "for", "from", "if", "import", "in",
            "none", "return", "then", "true", "try", "while", "with",
        ],
        CodeLanguage::Slash => &[
            "abstract", "async", "await", "break", "case", "catch", "class", "const", "continue",
            "do", "else", "enum", "export", "extends", "false", "final", "fn", "for",
            "function", "if", "impl", "import", "interface", "let", "match", "mod", "mut", "new",
            "null", "override", "package", "private", "protected", "pub", "return", "sealed",
            "static", "struct", "super", "switch", "this", "throw", "trait", "true", "type",
            "use", "var", "void", "while",
        ],
        CodeLanguage::Sql => &[
            "alter", "and", "as", "by", "create", "delete", "desc", "drop", "from", "group",
            "having", "insert", "into", "join", "limit", "not", "null", "on", "or", "order",
            "select", "set", "table", "update", "values", "where",
        ],
        CodeLanguage::Markup => &[
            "body", "button", "class", "div", "footer", "form", "head", "header", "html", "id",
            "input", "label", "link", "main", "meta", "nav", "script", "section", "span", "style",
            "title",
        ],
        CodeLanguage::Css => &[
            "align-items", "background", "border", "color", "display", "flex", "font-family",
            "font-size", "gap", "grid", "height", "justify-content", "margin", "padding",
            "position", "transition", "width",
        ],
        CodeLanguage::Plain => &[],
    }
}

#[derive(Clone, Copy)]
pub(super) enum CodeLanguage {
    Hash,
    Slash,
    Sql,
    Markup,
    Css,
    Plain,
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

fn language_for_filename(name: &str) -> CodeLanguage {
    let lower = name.to_ascii_lowercase();
    if HASH_FILENAMES.contains(&lower.as_str()) {
        return CodeLanguage::Hash;
    }
    if PLAIN_FILENAMES.contains(&lower.as_str()) {
        return CodeLanguage::Plain;
    }
    CodeLanguage::Plain
}

fn is_known_text_filename(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    HASH_FILENAMES.contains(&lower.as_str()) || PLAIN_FILENAMES.contains(&lower.as_str())
}

fn language_for_extension(ext: &str) -> Option<CodeLanguage> {
    match ext {
        "py" | "pyw" | "sh" | "bash" | "zsh" | "fish" | "toml" | "yaml" | "yml" | "rb"
        | "pl" | "pm" | "nix" | "ini" | "cfg" | "conf" | "env" => Some(CodeLanguage::Hash),
        "sql" => Some(CodeLanguage::Sql),
        "html" | "htm" | "xml" => Some(CodeLanguage::Markup),
        "css" | "scss" | "sass" | "less" => Some(CodeLanguage::Css),
        "md" | "txt" | "log" => Some(CodeLanguage::Plain),
        ext if CODE_EXTENSIONS.contains(&ext) => Some(CodeLanguage::Slash),
        _ => None,
    }
}

const CODE_EXTENSIONS: &[&str] = &[
    "rs", "rt", "dart", "js", "mjs", "cjs", "ts", "tsx", "jsx", "py", "pyw", "sh", "bash",
    "zsh", "fish", "json", "jsonc", "toml", "yaml", "yml", "md", "html", "htm", "css", "scss",
    "sass", "less", "sql", "xml", "go", "java", "kt", "kts", "swift", "c", "h", "hh", "hpp",
    "hxx", "cpp", "cc", "cxx", "cs", "php", "rb", "lua", "zig", "nix", "ini", "cfg", "conf",
    "env", "txt", "log", "pl", "pm", "ps1", "psm1", "bat", "cmd",
];

const HASH_FILENAMES: &[&str] = &[
    "dockerfile",
    "makefile",
    "justfile",
    "procfile",
    "rakefile",
    "jenkinsfile",
    "brewfile",
    ".gitignore",
    ".gitmodules",
    ".dockerignore",
    ".editorconfig",
    ".bashrc",
    ".zshrc",
    ".profile",
    ".bash_profile",
    ".env",
    ".env.local",
    ".env.development",
    ".env.production",
];

const PLAIN_FILENAMES: &[&str] = &["readme", "license", "copying", "authors", "notice", "todo"];

#[cfg(test)]
mod tests {
    use super::{language_for_path, supports_code_preview, CodeLanguage};
    use std::path::Path;

    #[test]
    fn supports_common_script_extensions() {
        assert!(supports_code_preview(Path::new("main.rs")));
        assert!(supports_code_preview(Path::new("app.dart")));
        assert!(supports_code_preview(Path::new("styles.css")));
        assert!(supports_code_preview(Path::new("build.gradle.kts")));
    }

    #[test]
    fn supports_known_text_filenames_without_extensions() {
        assert!(supports_code_preview(Path::new("Dockerfile")));
        assert!(supports_code_preview(Path::new(".gitignore")));
        assert!(supports_code_preview(Path::new(".env.production")));
    }

    #[test]
    fn infers_language_from_path() {
        assert!(matches!(
            language_for_path(Path::new("lib/main.dart")),
            CodeLanguage::Slash
        ));
        assert!(matches!(
            language_for_path(Path::new("styles/app.css")),
            CodeLanguage::Css
        ));
        assert!(matches!(
            language_for_path(Path::new("config/.env")),
            CodeLanguage::Hash
        ));
    }
}
