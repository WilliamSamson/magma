use std::{cell::RefCell, rc::Rc};

use gtk::{Box as GtkBox, Orientation, Stack, StackTransitionType, prelude::*};

use super::{Settings, about::build_about_page, sections::build_main_page};

type TitleSetter = Rc<dyn Fn(&str)>;
type CloseAction = Rc<dyn Fn()>;
type CloseActionSetter = Rc<dyn Fn(CloseAction)>;

pub(in crate::linux_terminal) fn build_settings_page(
    settings: Rc<RefCell<Settings>>,
    on_back: impl Fn() + 'static,
    on_title_change: impl Fn(&str) + 'static,
    on_close_action_change: impl Fn(CloseAction) + 'static,
    on_apply: impl Fn(&Settings) + 'static,
    on_clear_browser_data: impl Fn() + 'static,
) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.add_css_class("magma-settings-root");

    let page_stack = Stack::new();
    page_stack.set_vexpand(true);
    page_stack.set_hexpand(true);
    page_stack.set_transition_type(StackTransitionType::Crossfade);
    page_stack.set_transition_duration(160);

    bind_header_state(
        &page_stack,
        Rc::new(on_back),
        Rc::new(on_title_change),
        Rc::new(on_close_action_change),
    );

    let on_apply: Rc<dyn Fn(&Settings)> = Rc::new(on_apply);
    let on_clear_browser_data: Rc<dyn Fn()> = Rc::new(on_clear_browser_data);
    let main_page = build_main_page(&page_stack, &settings, on_apply, on_clear_browser_data);
    let about_page = build_about_page();

    page_stack.add_named(&main_page, Some("main"));
    page_stack.add_named(&about_page, Some("about"));
    page_stack.set_visible_child_name("main");

    root.append(&page_stack);
    root
}

fn bind_header_state(
    page_stack: &Stack,
    on_back: Rc<dyn Fn()>,
    set_title: TitleSetter,
    set_close_action: CloseActionSetter,
) {
    sync_header_state(page_stack, &on_back, &set_title, &set_close_action);
    let on_back_ref = on_back.clone();
    let title_ref = set_title.clone();
    let close_ref = set_close_action.clone();
    page_stack.connect_visible_child_name_notify(move |stack| {
        sync_header_state(stack, &on_back_ref, &title_ref, &close_ref);
    });
}

fn sync_header_state(
    page_stack: &Stack,
    on_back: &Rc<dyn Fn()>,
    set_title: &TitleSetter,
    set_close_action: &CloseActionSetter,
) {
    if page_stack.visible_child_name().as_deref() == Some("about") {
        set_title("About");
        let stack_ref = page_stack.clone();
        set_close_action(Rc::new(move || {
            stack_ref.set_visible_child_name("main");
        }));
        return;
    }

    set_title("Settings");
    let on_back = on_back.clone();
    set_close_action(Rc::new(move || {
        on_back();
    }));
}
