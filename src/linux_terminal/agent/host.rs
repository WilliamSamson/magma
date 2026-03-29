use std::{cell::Cell, cell::RefCell, collections::VecDeque, rc::Rc};

use gtk::{prelude::*, Box as GtkBox, Orientation, Overflow};

use super::build_agent_pane;
use crate::linux_terminal::settings::Settings;

#[derive(Clone)]
pub(in crate::linux_terminal) struct AgentPaneHost {
    root: GtkBox,
    loaded: Rc<Cell<bool>>,
    settings: Rc<RefCell<Settings>>,
    command_slot: Rc<RefCell<VecDeque<String>>>,
}

impl AgentPaneHost {
    pub(in crate::linux_terminal) fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(false);
        root.set_vexpand(true);
        root.set_width_request(0);
        root.set_overflow(Overflow::Hidden);

        Self {
            root,
            loaded: Rc::new(Cell::new(false)),
            settings,
            command_slot: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        let pane = build_agent_pane(self.settings.clone(), self.command_slot.clone());
        self.root.append(&pane);
    }

    pub(in crate::linux_terminal) fn dispatch_command(&self, command: &str) {
        self.command_slot
            .borrow_mut()
            .push_back(command.to_string());
    }
}
