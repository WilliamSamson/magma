use std::{cell::Cell, rc::Rc};

use gtk::{prelude::*, Box as GtkBox, Orientation, Overflow};
use webkit6::WebContext;

use super::{build_view_pane, CwdProvider};

#[derive(Clone)]
pub(in crate::linux_terminal) struct ViewPaneHost {
    root: GtkBox,
    // Rc<Cell<bool>> is enough because the viewer only needs one shared lazy-load flag on the GTK thread.
    loaded: Rc<Cell<bool>>,
    cwd_provider: CwdProvider,
    context: WebContext,
}

impl ViewPaneHost {
    pub(in crate::linux_terminal) fn new(cwd_provider: CwdProvider, context: WebContext) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_width_request(0);
        root.set_overflow(Overflow::Hidden);

        Self {
            root,
            loaded: Rc::new(Cell::new(false)),
            cwd_provider,
            context,
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        let pane = build_view_pane(self.cwd_provider.clone(), self.context.clone());
        self.root.append(&pane);
    }
}
