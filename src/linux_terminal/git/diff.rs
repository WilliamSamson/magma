use gtk::{
    pango, prelude::*, Box as GtkBox, Label, Orientation, Overflow,
};

use super::ops::{DiffHunk, DiffLineKind};

/// Build a diff view from parsed hunks. If `on_stage_hunk` is Some, stage/unstage buttons are shown per hunk.
pub(super) fn build_diff_widget(
    hunks: &[DiffHunk],
    on_stage_hunk: Option<&dyn Fn(usize)>,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-diff-root");
    root.set_hexpand(true);
    root.set_width_request(0);
    root.set_overflow(Overflow::Hidden);

    if hunks.is_empty() {
        let empty = Label::new(Some("no changes"));
        empty.add_css_class("magma-git-empty");
        empty.set_xalign(0.0);
        root.append(&empty);
        return root;
    }

    for hunk in hunks {
        let hunk_box = GtkBox::new(Orientation::Vertical, 0);
        hunk_box.add_css_class("magma-git-hunk");
        hunk_box.set_hexpand(true);
        hunk_box.set_width_request(0);
        hunk_box.set_overflow(Overflow::Hidden);

        // Hunk header row
        let header_row = GtkBox::new(Orientation::Horizontal, 4);
        header_row.add_css_class("magma-git-hunk-header");

        let header_label = Label::new(Some(&hunk.header));
        header_label.add_css_class("magma-git-hunk-header-text");
        header_label.set_xalign(0.0);
        header_label.set_hexpand(true);
        header_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        header_row.append(&header_label);

        let _ = on_stage_hunk; // reserved for future hunk-level staging

        hunk_box.append(&header_row);

        // Diff lines
        for line in &hunk.lines {
            let line_label = Label::new(Some(&line.content));
            line_label.set_xalign(0.0);
            line_label.set_selectable(true);
            line_label.set_hexpand(true);
            line_label.set_width_request(0);
            line_label.set_wrap(true);
            line_label.set_wrap_mode(pango::WrapMode::Char);
            line_label.set_ellipsize(pango::EllipsizeMode::None);
            line_label.add_css_class("magma-git-diff-line");
            line_label.add_css_class(match line.kind {
                DiffLineKind::Added => "magma-git-line-added",
                DiffLineKind::Removed => "magma-git-line-removed",
                DiffLineKind::Context => "magma-git-line-context",
                DiffLineKind::Header => "magma-git-line-header",
            });
            hunk_box.append(&line_label);
        }

        root.append(&hunk_box);
    }

    root
}

/// Build a simple diff stat summary from commit.
pub(super) fn build_diff_stat(stat_text: &str) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-git-diff-stat");

    for line in stat_text.lines() {
        let label = Label::new(Some(line));
        label.set_xalign(0.0);
        label.add_css_class("magma-git-diff-stat-line");
        root.append(&label);
    }

    root
}
