use std::{cell::RefCell, rc::Rc};

use gtk::{
    Box as GtkBox, Entry, Orientation, ScrolledWindow, Stack, StackTransitionType, glib, prelude::*,
};

use crate::linux_terminal::meta::APP_VERSION;

use super::{
    Settings,
    browser::build_browser_section,
    search::{SearchSection, bind_settings_search},
    settings_path,
    terminal::{build_appearance_section, build_terminal_section, preview_settings},
    widgets::{
        action_row, body_copy, info_row, nav_button, nav_search_entry, section_label, switch_row,
        text_row,
    },
};
const SEARCH_EMPTY_ID: &str = "search-empty";

type SectionBuilder = fn(&GtkBox, &Rc<RefCell<Settings>>, &Rc<dyn Fn(&Settings)>);

struct SectionView {
    id: &'static str,
    title: &'static str,
    content: GtkBox,
    panel: GtkBox,
}

pub(super) fn build_main_page(
    page_stack: &Stack,
    settings: &Rc<RefCell<Settings>>,
    on_apply: Rc<dyn Fn(&Settings)>,
    on_clear_browser_data: Rc<dyn Fn()>,
) -> GtkBox {
    let section_views =
        build_section_views(page_stack, settings, &on_apply, &on_clear_browser_data);
    let detail_stack = build_section_stack(&section_views);
    let (nav, search_entry, search_sections) = build_section_nav(&detail_stack, &section_views);
    bind_settings_search(
        &search_entry,
        &detail_stack,
        SEARCH_EMPTY_ID,
        search_sections,
    );

    let root = GtkBox::new(Orientation::Horizontal, 10);
    root.add_css_class("magma-settings-main");
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.append(&nav);
    root.append(&detail_stack);
    root
}

fn build_section_nav(
    detail_stack: &Stack,
    section_views: &[SectionView],
) -> (GtkBox, Entry, Vec<SearchSection>) {
    let nav = GtkBox::new(Orientation::Vertical, 10);
    nav.add_css_class("magma-settings-nav");
    nav.set_size_request(235, -1);
    nav.set_vexpand(true);

    let search_entry = nav_search_entry();
    nav.append(&search_entry);

    let search_sections = section_views
        .iter()
        .map(|view| {
            let button = nav_button(view.title);
            let section_id = view.id;
            let stack_ref = detail_stack.clone();
            button.connect_clicked(move |_| {
                stack_ref.set_visible_child_name(section_id);
            });
            nav.append(&button);
            SearchSection::new(view.id, view.title, &button, &view.content)
        })
        .collect::<Vec<SearchSection>>();

    sync_nav_selection(detail_stack, &search_sections);
    let sections_ref = search_sections.clone();
    detail_stack.connect_visible_child_name_notify(move |stack| {
        sync_nav_selection(stack, &sections_ref);
    });

    (nav, search_entry, search_sections)
}

fn build_section_views(
    page_stack: &Stack,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
    on_clear_browser_data: &Rc<dyn Fn()>,
) -> Vec<SectionView> {
    let appearance = build_settings_section_view(
        "appearance",
        "appearance",
        build_appearance_section,
        settings,
        on_apply,
    );
    let terminal = build_settings_section_view(
        "terminal",
        "terminal",
        build_terminal_section,
        settings,
        on_apply,
    );
    let browser = build_browser_view(settings, on_apply, on_clear_browser_data);
    let shell =
        build_settings_section_view("shell", "shell", build_shell_section, settings, on_apply);
    let logr = build_settings_section_view("logr", "logr", build_logr_section, settings, on_apply);
    let about = build_about_view(page_stack);

    vec![appearance, terminal, browser, shell, logr, about]
}

fn build_section_stack(section_views: &[SectionView]) -> Stack {
    let detail_stack = Stack::new();
    detail_stack.add_css_class("magma-settings-detail-stack");
    detail_stack.set_hexpand(true);
    detail_stack.set_vexpand(true);
    detail_stack.set_transition_type(StackTransitionType::Crossfade);
    detail_stack.set_transition_duration(160);

    for view in section_views {
        detail_stack.add_named(&view.panel, Some(view.id));
    }
    detail_stack.add_named(&build_empty_search_panel(), Some(SEARCH_EMPTY_ID));
    detail_stack.set_visible_child_name("appearance");
    detail_stack
}

fn build_settings_section_view(
    id: &'static str,
    title: &'static str,
    builder: SectionBuilder,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) -> SectionView {
    let content = build_section_content();
    builder(&content, settings, on_apply);
    SectionView {
        id,
        title,
        panel: wrap_section_content(&content),
        content,
    }
}

fn build_browser_view(
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
    on_clear_browser_data: &Rc<dyn Fn()>,
) -> SectionView {
    let content = build_section_content();
    build_browser_section(&content, settings, on_apply, on_clear_browser_data);
    SectionView {
        id: "browser",
        title: "browser",
        panel: wrap_section_content(&content),
        content,
    }
}

fn build_about_view(page_stack: &Stack) -> SectionView {
    let content = build_section_content();
    build_about_section(&content, page_stack);
    SectionView {
        id: "about",
        title: "about",
        panel: wrap_section_content(&content),
        content,
    }
}

fn build_section_content() -> GtkBox {
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.add_css_class("magma-settings-content");
    content
}

fn wrap_section_content(content: &GtkBox) -> GtkBox {
    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_hexpand(true);
    scroller.set_child(Some(content));

    let panel = GtkBox::new(Orientation::Vertical, 0);
    panel.add_css_class("magma-settings-detail");
    panel.set_vexpand(true);
    panel.set_hexpand(true);
    panel.append(&scroller);
    panel
}

fn build_empty_search_panel() -> GtkBox {
    let content = GtkBox::new(Orientation::Vertical, 10);
    content.add_css_class("magma-settings-empty");

    let title = section_label("no results");
    let copy =
        body_copy("Try another search term or clear the search field to see every setting again.");
    content.append(&title);
    content.append(&copy);
    wrap_section_content(&content)
}

fn sync_nav_selection(detail_stack: &Stack, sections: &[SearchSection]) {
    let active = detail_stack.visible_child_name();
    for section in sections {
        if active.as_deref() == Some(section.id) {
            section.button.add_css_class("active");
            continue;
        }
        section.button.remove_css_class("active");
    }
}

fn build_shell_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("shell"));
    let shell_entry = text_row(
        content,
        "shell command",
        "the executable shell launched when a new magma session starts.",
        &settings.borrow().shell,
    );
    let shell_settings = settings.clone();
    let shell_apply = on_apply.clone();
    shell_entry.connect_changed(move |entry| {
        shell_settings.borrow_mut().shell = entry.text().to_string();
        preview_settings(&shell_settings, &shell_apply);
    });
}
fn build_logr_section(
    content: &GtkBox,
    settings: &Rc<RefCell<Settings>>,
    on_apply: &Rc<dyn Fn(&Settings)>,
) {
    content.append(&section_label("logr"));
    let logr_switch = switch_row(
        content,
        "panel open on start",
        "automatically reveal the logr and web pane when magma boots.",
        settings.borrow().logr_panel_open,
    );
    let logr_settings = settings.clone();
    let logr_apply = on_apply.clone();
    logr_switch.connect_state_set(move |_, active| {
        logr_settings.borrow_mut().logr_panel_open = active;
        preview_settings(&logr_settings, &logr_apply);
        glib::Propagation::Proceed
    });
}

fn build_about_section(content: &GtkBox, page_stack: &Stack) {
    content.append(&section_label("about"));
    info_row(content, "version", APP_VERSION);
    info_row(content, "config", &settings_path().display().to_string());

    let about_button = action_row(
        content,
        "magma",
        "view credits, licenses, and core engine details.",
        "open",
    );
    let stack_ref = page_stack.clone();
    about_button.connect_clicked(move |_| {
        stack_ref.set_visible_child_name("about");
    });
}
