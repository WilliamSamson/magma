use std::{
    path::Path,
    process::Command,
    rc::Rc,
};

use gtk::{
    gio, prelude::*, Box as GtkBox, Button, Label, Orientation, Picture, ScrolledWindow, Stack,
};
use webkit6::{prelude::*, WebContext, WebView};

use super::{
    docx,
    editor::{build_editor, hide_code_actions, show_code_preview, EditorWidgets},
    files::{format_size, kind_label, FileKind, ViewerFile},
    ui::build_empty_state,
    ViewState,
};

pub(super) struct PreviewWidgets {
    pub(super) root: GtkBox,
    pub(super) stack: Stack,
    pub(super) editor: EditorWidgets,
    picture: Picture,
    web_view: WebView,
    pub(super) title: Label,
    pub(super) meta: Label,
    info_title: Label,
    info_body: Label,
    pub(super) open_button: Button,
}

pub(super) fn build_preview(context: WebContext) -> PreviewWidgets {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-view-preview");
    root.set_hexpand(true);
    root.set_vexpand(true);

    let chrome = GtkBox::new(Orientation::Horizontal, 8);
    chrome.add_css_class("obsidian-view-preview-chrome");
    chrome.set_hexpand(true);
    let title_box = GtkBox::new(Orientation::Vertical, 1);
    title_box.set_hexpand(true);
    let title = Label::new(Some("viewer idle"));
    title.add_css_class("obsidian-view-preview-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    let meta = Label::new(Some("select a file from the list"));
    meta.add_css_class("obsidian-view-preview-meta");
    meta.set_xalign(0.0);
    meta.set_hexpand(true);
    meta.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_box.append(&title);
    title_box.append(&meta);

    let open_button = Button::with_label("Open external");
    open_button.add_css_class("obsidian-view-preview-button");
    open_button.add_css_class("obsidian-view-preview-button-secondary");
    open_button.set_sensitive(false);
    let action_box = GtkBox::new(Orientation::Horizontal, 8);
    action_box.add_css_class("obsidian-view-preview-actions");
    action_box.append(&open_button);
    chrome.append(&title_box);
    chrome.append(&action_box);

    let stack = Stack::new();
    stack.add_css_class("obsidian-view-preview-stack");
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let empty = build_empty_state("dialog-information-symbolic", "no preview file");
    stack.add_named(&empty, Some("empty"));

    let editor = build_editor();
    stack.add_named(&editor.root, Some("code"));

    let picture = Picture::new();
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    let image_scroll = ScrolledWindow::new();
    image_scroll.add_css_class("obsidian-view-preview-surface");
    image_scroll.set_child(Some(&picture));
    stack.add_named(&image_scroll, Some("image"));

    let web_view = WebView::builder().web_context(&context).build();
    web_view.set_hexpand(true);
    web_view.set_vexpand(true);
    stack.add_named(&web_view, Some("document"));

    let info_box = GtkBox::new(Orientation::Vertical, 8);
    info_box.add_css_class("obsidian-view-info");
    let info_title = Label::new(Some("document"));
    info_title.add_css_class("obsidian-view-info-title");
    info_title.set_xalign(0.0);
    let info_body = Label::new(None);
    info_body.add_css_class("obsidian-view-info-body");
    info_body.set_wrap(true);
    info_body.set_xalign(0.0);
    info_box.append(&info_title);
    info_box.append(&info_body);
    stack.add_named(&info_box, Some("info"));
    stack.set_visible_child_name("empty");

    action_box.append(&editor.reload_button);
    action_box.append(&editor.save_button);
    root.append(&chrome);
    root.append(&stack);

    PreviewWidgets {
        root,
        stack,
        editor,
        picture,
        web_view,
        title,
        meta,
        info_title,
        info_body,
        open_button,
    }
}

pub(super) fn bind_open_external(state: &Rc<ViewState>) {
    let state_ref = state.clone();
    state.widgets.preview.open_button.connect_clicked(move |_| {
        if let Some(path) = state_ref.model.borrow().selected_file.clone() {
            let _ = Command::new("xdg-open").arg(path).spawn();
        }
    });
}

pub(super) fn select_file(state: &Rc<ViewState>, file: &ViewerFile) {
    state.model.borrow_mut().selected_file = Some(file.path.clone());
    update_preview_header(&state.widgets.preview, file);
    hide_code_actions(&state.widgets.preview);
    match file.kind {
        FileKind::Code => {
            if let Err(error) = show_code_preview(state, &state.widgets.preview, file) {
                show_error_preview(&state.widgets.preview, file, &error);
            }
        }
        FileKind::Image => {
            state
                .widgets
                .preview
                .picture
                .set_file(Some(&gio::File::for_path(&file.path)));
            state.widgets.preview.stack.set_visible_child_name("image");
        }
        FileKind::Pdf => {
            state
                .widgets
                .preview
                .web_view
                .load_uri(&gio::File::for_path(&file.path).uri());
            state.widgets.preview.stack.set_visible_child_name("document");
        }
        FileKind::Docx => show_docx_preview(&state.widgets.preview, file),
        FileKind::Office => show_info_preview(&state.widgets.preview, file),
    }
}

pub(super) fn set_empty_preview(preview: &PreviewWidgets, message: &str) {
    hide_code_actions(preview);
    preview.title.set_text("viewer idle");
    preview.meta.set_text(message);
    preview.open_button.set_sensitive(false);

    if let Some(lbl) = find_empty_label(&preview.stack) {
        lbl.set_text(message);
    }
    preview.stack.set_visible_child_name("empty");
}

fn find_empty_label(stack: &Stack) -> Option<Label> {
    let empty = stack.child_by_name("empty")?;
    let mut child = empty.first_child();
    while let Some(w) = child {
        if let Some(lbl) = w.downcast_ref::<Label>() {
            if lbl.has_css_class("obsidian-view-empty-text") {
                return Some(lbl.clone());
            }
        }
        child = w.next_sibling();
    }
    None
}

fn show_info_preview(preview: &PreviewWidgets, file: &ViewerFile) {
    preview.info_title.set_text(&file.name);
    preview.info_body.set_text(&format!(
        "{} document\n{}\n{}\nEmbedded preview is not available yet for this file type.",
        office_label(&file.path),
        format_size(file.size_bytes),
        file.path.display()
    ));
    preview.stack.set_visible_child_name("info");
}

fn show_error_preview(preview: &PreviewWidgets, file: &ViewerFile, error: &str) {
    preview.info_title.set_text(&file.name);
    preview.info_body.set_text(&format!(
        "{} preview unavailable\n{}\n{}\n{}",
        kind_label(file.kind),
        format_size(file.size_bytes),
        file.path.display(),
        error
    ));
    preview.stack.set_visible_child_name("info");
}

fn show_docx_preview(preview: &PreviewWidgets, file: &ViewerFile) {
    match docx::render_docx_html(&file.path) {
        Ok(html) => {
            preview.web_view.load_html(&html, None);
            preview.stack.set_visible_child_name("document");
        }
        Err(error) => {
            preview.info_title.set_text(&file.name);
            preview.info_body.set_text(&format!(
                "docx preview unavailable\n{}\n{}\n{}",
                format_size(file.size_bytes),
                file.path.display(),
                error
            ));
            preview.stack.set_visible_child_name("info");
        }
    }
}

fn update_preview_header(preview: &PreviewWidgets, file: &ViewerFile) {
    preview.title.set_text(&file.name);
    preview.meta.set_text(&format!(
        "{} · {}",
        kind_label(file.kind),
        format_size(file.size_bytes)
    ));
    preview.open_button.set_sensitive(true);
}

fn office_label(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
    {
        Some(ext) if ext.starts_with("ppt") => "presentation",
        _ => "document",
    }
}
