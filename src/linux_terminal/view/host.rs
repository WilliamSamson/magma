use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::{prelude::*, Box as GtkBox, Orientation, Overflow};
use webkit6::WebContext;

use super::{build_view_pane, CwdProvider};

/// Shared slot for requesting the view pane to open a specific file.
pub(super) type OpenFileSlot = Rc<RefCell<Option<PathBuf>>>;

#[derive(Clone)]
pub(in crate::linux_terminal) struct ViewPaneHost {
    root: GtkBox,
    loaded: Rc<Cell<bool>>,
    cwd_provider: CwdProvider,
    context: WebContext,
    open_file_slot: OpenFileSlot,
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
            open_file_slot: Rc::new(RefCell::new(None)),
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        let pane = build_view_pane(
            self.cwd_provider.clone(),
            self.context.clone(),
            self.open_file_slot.clone(),
        );
        self.root.append(&pane);
    }

    /// Request the view pane to open a specific file.
    /// The pane will pick this up on its next refresh cycle.
    pub(in crate::linux_terminal) fn open_file(&self, path: &Path) {
        *self.open_file_slot.borrow_mut() = Some(path.to_path_buf());
    }
}
