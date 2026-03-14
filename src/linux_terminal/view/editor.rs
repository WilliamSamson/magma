use std::{
    cell::Cell,
    fs,
    rc::Rc,
};

use gtk::{
    prelude::*, Button, PolicyType, ScrolledWindow, TextBuffer, TextView, WrapMode,
};

use super::{
    code::{comment_tokens, keywords, language_for_path, language_for_text, CodeLanguage},
    files::{format_size, kind_label, FileKind, ViewerFile},
    preview::PreviewWidgets,
    ViewState,
};

pub(super) struct EditorWidgets {
    pub(super) root: ScrolledWindow,
    pub(super) save_button: Button,
    pub(super) reload_button: Button,
    buffer: TextBuffer,
    // Rc<Cell<bool>> avoids re-entrant highlight/save state during programmatic buffer updates on the GTK thread.
    loading: Rc<Cell<bool>>,
    // Rc<Cell<bool>> is enough for a one-bit debounce gate before running the idle highlighter.
    highlight_pending: Rc<Cell<bool>>,
}

pub(super) fn build_editor() -> EditorWidgets {
    let buffer = TextBuffer::new(None);
    install_tags(&buffer);

    let view = TextView::with_buffer(&buffer);
    view.add_css_class("obsidian-view-code");
    view.set_hexpand(true);
    view.set_vexpand(true);
    view.set_monospace(true);
    view.set_wrap_mode(WrapMode::WordChar);
    view.set_top_margin(14);
    view.set_bottom_margin(18);
    view.set_left_margin(16);
    view.set_right_margin(16);

    let root = ScrolledWindow::new();
    root.add_css_class("obsidian-view-code-scroller");
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_policy(PolicyType::Never, PolicyType::Automatic);
    root.set_child(Some(&view));

    let reload_button = Button::with_label("Reload");
    reload_button.add_css_class("obsidian-view-preview-button");
    reload_button.add_css_class("obsidian-view-preview-button-secondary");
    reload_button.set_visible(false);
    reload_button.set_sensitive(false);

    let save_button = Button::with_label("Save");
    save_button.add_css_class("obsidian-view-preview-button");
    save_button.add_css_class("obsidian-view-preview-button-primary");
    save_button.set_visible(false);
    save_button.set_sensitive(false);

    EditorWidgets {
        root,
        save_button,
        reload_button,
        buffer,
        loading: Rc::new(Cell::new(false)),
        highlight_pending: Rc::new(Cell::new(false)),
    }
}

pub(super) fn bind_editor_actions(state: &Rc<ViewState>) {
    let buffer = state.widgets.preview.editor.buffer.clone();
    let state_ref = state.clone();
    let loading = state.widgets.preview.editor.loading.clone();
    let pending = state.widgets.preview.editor.highlight_pending.clone();
    buffer.connect_changed(move |buffer| {
        if loading.get() {
            return;
        }
        schedule_highlight(buffer, pending.clone());
        update_dirty_state(&state_ref);
    });

    let buffer = state.widgets.preview.editor.buffer.clone();
    let state_ref = state.clone();
    buffer.connect_modified_changed(move |_| update_dirty_state(&state_ref));

    let state_ref = state.clone();
    state
        .widgets
        .preview
        .editor
        .reload_button
        .connect_clicked(move |_| reload_current_code_file(&state_ref));

    let state_ref = state.clone();
    state
        .widgets
        .preview
        .editor
        .save_button
        .connect_clicked(move |_| save_current_code_file(&state_ref));
}

pub(super) fn show_code_preview(
    state: &Rc<ViewState>,
    preview: &PreviewWidgets,
    file: &ViewerFile,
) -> Result<(), String> {
    let text = fs::read_to_string(&file.path)
        .map_err(|error| format!("code preview unavailable: {error}"))?;

    preview.editor.loading.set(true);
    preview.editor.buffer.set_text(&text);
    preview.editor.buffer.set_modified(false);
    apply_highlighting(&preview.editor.buffer, language_for_path(&file.path));
    preview.editor.loading.set(false);

    preview.stack.set_visible_child_name("code");
    preview.editor.reload_button.set_visible(true);
    preview.editor.save_button.set_visible(true);
    preview.editor.reload_button.set_sensitive(true);
    preview.editor.save_button.set_sensitive(false);
    preview
        .meta
        .set_text(&format!("{} · {}", kind_label(file.kind), format_size(file.size_bytes)));
    state.model.borrow_mut().selected_file = Some(file.path.clone());
    Ok(())
}

pub(super) fn hide_code_actions(preview: &PreviewWidgets) {
    preview.editor.reload_button.set_visible(false);
    preview.editor.reload_button.set_sensitive(false);
    preview.editor.save_button.set_visible(false);
    preview.editor.save_button.set_sensitive(false);
}

fn reload_current_code_file(state: &Rc<ViewState>) {
    let file = current_code_file(state);
    let Some(file) = file else {
        return;
    };
    if let Err(error) = show_code_preview(state, &state.widgets.preview, &file) {
        state
            .widgets
            .preview
            .meta
            .set_text(&format!("reload failed · {error}"));
    }
}

fn save_current_code_file(state: &Rc<ViewState>) {
    let file = current_code_file(state);
    let Some(file) = file else {
        return;
    };

    let buffer = &state.widgets.preview.editor.buffer;
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let text = buffer.text(&start, &end, true);
    match fs::write(&file.path, text.as_str()) {
        Ok(()) => {
            buffer.set_modified(false);
            state.widgets.preview.meta.set_text(&format!(
                "{} · {}",
                kind_label(file.kind),
                format_size(file.size_bytes)
            ));
        }
        Err(error) => {
            state
                .widgets
                .preview
                .meta
                .set_text(&format!("save failed · {error}"));
        }
    }
}

fn update_dirty_state(state: &Rc<ViewState>) {
    let file = current_code_file(state);
    let Some(file) = file else {
        return;
    };

    let modified = state.widgets.preview.editor.buffer.is_modified();
    state
        .widgets
        .preview
        .editor
        .save_button
        .set_sensitive(modified);
    state.widgets.preview.meta.set_text(&format!(
        "{} · {}{}",
        kind_label(file.kind),
        format_size(file.size_bytes),
        if modified { " · edited" } else { "" }
    ));
}

fn current_code_file(state: &Rc<ViewState>) -> Option<ViewerFile> {
    let model = state.model.borrow();
    let selected = model.selected_file.as_ref()?;
    let file = model.files.iter().find(|file| &file.path == selected)?;
    (file.kind == FileKind::Code).then_some(file.clone())
}

fn schedule_highlight(buffer: &TextBuffer, pending: Rc<Cell<bool>>) {
    if pending.replace(true) {
        return;
    }

    let buffer_ref = buffer.clone();
    gtk::glib::idle_add_local_once(move || {
        pending.set(false);
        let start = buffer_ref.start_iter();
        let end = buffer_ref.end_iter();
        let text = buffer_ref.text(&start, &end, true).to_string();
        apply_highlighting(&buffer_ref, language_for_text(&text));
    });
}

fn install_tags(buffer: &TextBuffer) {
    let _ = buffer.create_tag(Some("code-keyword"), &[("foreground", &"#ff6b6b"), ("weight", &700)]);
    let _ = buffer.create_tag(Some("code-string"), &[("foreground", &"#8bd450")]);
    let _ = buffer.create_tag(Some("code-comment"), &[("foreground", &"#7e8294")]);
    let _ = buffer.create_tag(Some("code-number"), &[("foreground", &"#ffbd2e")]);
}

fn apply_highlighting(buffer: &TextBuffer, language: CodeLanguage) {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let text = buffer.text(&start, &end, true).to_string();
    buffer.remove_all_tags(&start, &end);

    let lines: Vec<&str> = text.lines().collect();
    let mut offset = 0usize;
    for (index, line) in lines.iter().enumerate() {
        highlight_line(buffer, line, offset, language);
        offset += line.chars().count();
        if index + 1 < lines.len() {
            offset += 1;
        }
    }
}

fn highlight_line(buffer: &TextBuffer, line: &str, line_offset: usize, language: CodeLanguage) {
    let comment_start = find_comment_start(line, language);
    let code_segment = &line[..comment_start.unwrap_or(line.len())];

    highlight_strings(buffer, code_segment, line_offset);
    highlight_numbers(buffer, code_segment, line_offset);
    highlight_keywords(buffer, code_segment, line_offset, language);

    if let Some(start) = comment_start {
        apply_tag(buffer, "code-comment", line_offset + char_count(&line[..start]), line_offset + line.chars().count());
    }
}

fn highlight_strings(buffer: &TextBuffer, text: &str, line_offset: usize) {
    let mut in_string = None;
    let mut start_char = 0usize;
    for (idx, ch) in text.chars().enumerate() {
        match in_string {
            None if ch == '"' || ch == '\'' => {
                in_string = Some(ch);
                start_char = idx;
            }
            Some(quote) if ch == quote => {
                apply_tag(buffer, "code-string", line_offset + start_char, line_offset + idx + 1);
                in_string = None;
            }
            _ => {}
        }
    }
}

fn highlight_numbers(buffer: &TextBuffer, text: &str, line_offset: usize) {
    let mut start = None;
    for (idx, ch) in text.chars().enumerate() {
        if ch.is_ascii_digit() && start.is_none() {
            start = Some(idx);
        } else if !ch.is_ascii_digit() {
            if let Some(number_start) = start.take() {
                apply_tag(buffer, "code-number", line_offset + number_start, line_offset + idx);
            }
        }
    }
    if let Some(number_start) = start {
        apply_tag(
            buffer,
            "code-number",
            line_offset + number_start,
            line_offset + text.chars().count(),
        );
    }
}

fn highlight_keywords(buffer: &TextBuffer, text: &str, line_offset: usize, language: CodeLanguage) {
    let keywords = keywords(language);
    let mut word = String::new();
    let mut start = 0usize;
    for (idx, ch) in text.chars().enumerate() {
        if ch.is_alphanumeric() || ch == '_' {
            if word.is_empty() {
                start = idx;
            }
            word.push(ch);
            continue;
        }

        if keywords.contains(&word.as_str()) {
            apply_tag(buffer, "code-keyword", line_offset + start, line_offset + idx);
        }
        word.clear();
    }

    if keywords.contains(&word.as_str()) {
        apply_tag(
            buffer,
            "code-keyword",
            line_offset + start,
            line_offset + text.chars().count(),
        );
    }
}

fn apply_tag(buffer: &TextBuffer, tag_name: &str, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let start_iter = buffer.iter_at_offset(start as i32);
    let end_iter = buffer.iter_at_offset(end as i32);
    buffer.apply_tag_by_name(tag_name, &start_iter, &end_iter);
}

fn find_comment_start(line: &str, language: CodeLanguage) -> Option<usize> {
    comment_tokens(language)
        .iter()
        .filter_map(|token| line.find(token))
        .min()
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}
