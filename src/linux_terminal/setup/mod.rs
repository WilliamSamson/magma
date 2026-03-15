mod checkpoint;
mod controls;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{
    prelude::*,
    Align, Box as GtkBox, Button, Label, Orientation, Stack, StackTransitionType,
};

use super::settings::Settings;

const STEP_COUNT: u32 = 3;

pub(super) fn load_checkpoint(initial_settings: &Settings) -> (Settings, u32) {
    checkpoint::load(initial_settings)
}

pub(super) fn clear_checkpoint() {
    checkpoint::clear();
}

pub(in crate::linux_terminal) fn build_setup_page(
    initial_settings: &Settings,
    initial_step: u32,
    on_continue: impl Fn(Settings) + 'static,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("magma-setup-root");
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_halign(Align::Center);
    root.set_valign(Align::Center);

    // Rc<RefCell<Settings>> keeps one editable setup draft shared by step callbacks on the GTK thread.
    let draft = Rc::new(RefCell::new(initial_settings.clone()));
    // Rc<Cell<u32>> shares the active step index across navigation callbacks without RefCell borrow overhead.
    let step_index = Rc::new(Cell::new(initial_step.min(STEP_COUNT.saturating_sub(1))));
    let persist_checkpoint: Rc<dyn Fn()> = {
        let draft = draft.clone();
        let step_index = step_index.clone();
        Rc::new(move || {
            checkpoint::save(&draft.borrow(), step_index.get());
        })
    };

    let shell = GtkBox::new(Orientation::Vertical, 0);
    shell.add_css_class("magma-setup-shell");
    shell.set_halign(Align::Center);
    shell.set_valign(Align::Center);

    shell.append(&controls::build_topbar());

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.add_css_class("magma-setup-body");

    let hero = controls::build_hero();
    let (progress, step_markers) = controls::build_progress();
    let page_stack = build_page_stack(&draft, &persist_checkpoint);
    let footer = GtkBox::new(Orientation::Horizontal, 10);
    footer.add_css_class("magma-setup-footer");

    let (back_button, _) = controls::build_nav_button(
        "back",
        "go-previous-symbolic",
        true,
        "magma-setup-secondary",
    );
    let (next_button, next_button_label) = controls::build_nav_button(
        "next",
        "go-next-symbolic",
        false,
        "magma-setup-action",
    );
    let footer_spacer = GtkBox::new(Orientation::Horizontal, 0);
    footer_spacer.set_hexpand(true);

    footer.append(&back_button);
    footer.append(&footer_spacer);
    footer.append(&next_button);

    body.append(&hero);
    body.append(&progress);
    body.append(&page_stack);
    body.append(&footer);
    shell.append(&body);
    root.append(&shell);

    bind_setup_navigation(
        NavigationWidgets {
            back_button: back_button.clone(),
            next_button: next_button.clone(),
            next_button_label: next_button_label.clone(),
        },
        &page_stack,
        step_markers,
        step_index,
        draft,
        persist_checkpoint,
        Rc::new(on_continue),
    );

    root
}

#[derive(Clone)]
struct NavigationWidgets {
    back_button: Button,
    next_button: Button,
    next_button_label: Label,
}

fn build_page_stack(draft: &Rc<RefCell<Settings>>, on_checkpoint: &Rc<dyn Fn()>) -> Stack {
    let page_stack = Stack::new();
    page_stack.add_css_class("magma-setup-pages");
    page_stack.set_transition_type(StackTransitionType::SlideLeftRight);
    page_stack.set_transition_duration(180);
    page_stack.add_named(&controls::build_runtime_step(draft, on_checkpoint), Some("runtime"));
    page_stack.add_named(
        &controls::build_workspace_step(draft, on_checkpoint),
        Some("workspace"),
    );
    page_stack.add_named(
        &controls::build_appearance_step(draft, on_checkpoint),
        Some("appearance"),
    );
    page_stack.set_visible_child_name("runtime");
    page_stack
}

fn bind_setup_navigation(
    nav: NavigationWidgets,
    page_stack: &Stack,
    step_markers: Vec<GtkBox>,
    step_index: Rc<Cell<u32>>,
    draft: Rc<RefCell<Settings>>,
    on_checkpoint: Rc<dyn Fn()>,
    on_continue: Rc<dyn Fn(Settings)>,
) {
    sync_step_ui(
        &step_index,
        page_stack,
        &nav.back_button,
        &nav.next_button,
        &nav.next_button_label,
        &step_markers,
    );

    {
        let step_index = step_index.clone();
        let page_stack = page_stack.clone();
        let back_click = nav.back_button.clone();
        let back_sync = nav.back_button.clone();
        let next_sync = nav.next_button.clone();
        let next_label_sync = nav.next_button_label.clone();
        let step_markers = step_markers.clone();
        let on_checkpoint = on_checkpoint.clone();
        back_click.connect_clicked(move |_| {
            let next_step = step_index.get().saturating_sub(1);
            step_index.set(next_step);
            on_checkpoint();
            sync_step_ui(
                &step_index,
                &page_stack,
                &back_sync,
                &next_sync,
                &next_label_sync,
                &step_markers,
            );
        });
    }

    {
        let step_index = step_index.clone();
        let draft = draft.clone();
        let on_checkpoint = on_checkpoint.clone();
        let on_continue = on_continue.clone();
        let page_stack = page_stack.clone();
        let next_click = nav.next_button.clone();
        let back_sync = nav.back_button.clone();
        let next_sync = nav.next_button.clone();
        let next_label_sync = nav.next_button_label.clone();
        next_click.connect_clicked(move |_| {
            let current = step_index.get();
            if current + 1 >= STEP_COUNT {
                on_continue(draft.borrow().clone());
                return;
            }

            step_index.set(current + 1);
            on_checkpoint();
            sync_step_ui(
                &step_index,
                &page_stack,
                &back_sync,
                &next_sync,
                &next_label_sync,
                &step_markers,
            );
        });
    }
}

fn sync_step_ui(
    step_index: &Rc<Cell<u32>>,
    page_stack: &Stack,
    back_button: &Button,
    next_button: &Button,
    next_button_label: &Label,
    step_markers: &[GtkBox],
) {
    let current = step_index.get();
    page_stack.set_visible_child_name(step_name(current));
    back_button.set_visible(current > 0);
    back_button.set_sensitive(current > 0);
    next_button_label.set_text(if current + 1 >= STEP_COUNT {
        "enter workspace"
    } else {
        "next"
    });
    next_button.set_sensitive(true);

    for (index, marker) in step_markers.iter().enumerate() {
        if index as u32 == current {
            marker.add_css_class("active");
        } else {
            marker.remove_css_class("active");
        }
    }
}

fn step_name(index: u32) -> &'static str {
    match index {
        1 => "workspace",
        2 => "appearance",
        _ => "runtime",
    }
}

pub(super) fn setup_label(text: &str, class_name: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class_name);
    label.set_xalign(0.0);
    label.set_selectable(false);
    label
}
