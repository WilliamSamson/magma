use std::{cell::Cell, rc::Rc};

use gtk::{Box as GtkBox, Orientation, Overflow, prelude::*};

use super::{CwdProvider, build_git_pane};

#[derive(Clone)]
pub(in crate::linux_terminal) struct GitPaneHost {
    root: GtkBox,
    loaded: Rc<Cell<bool>>,
    cwd_provider: CwdProvider,
}

impl GitPaneHost {
    pub(in crate::linux_terminal) fn new(cwd_provider: CwdProvider) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_width_request(0);
        root.set_overflow(Overflow::Hidden);

        Self {
            root,
            loaded: Rc::new(Cell::new(false)),
            cwd_provider,
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        let pane = build_git_pane(self.cwd_provider.clone());
        self.root.append(&pane);
    }
}
