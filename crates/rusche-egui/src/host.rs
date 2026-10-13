//! Frame-local [`egui::Ui`]. Widget natives borrow it through a pointer slot so a
//! Scheme procedure can re-enter the interpreter without holding a `RefCell` guard
//! across the call.

use std::cell::{Cell, RefCell};

use rusche::{ErrorKind, EvalContext, EvalError, EvalResult, Expr};

pub(crate) struct Host {
    ui: Cell<*mut egui::Ui>,
    frame: Cell<u64>,
    /// Reused by headless frames so panel size and editor focus persist.
    ctx: RefCell<egui::Context>,
    last_response: RefCell<Option<egui::Response>>,
    /// `true` when the latest [`crate::Egui::run_frame`] asked egui to repaint immediately.
    repaint: Cell<bool>,
    time: Cell<f64>,
}

impl Host {
    pub(crate) fn new() -> Self {
        Self {
            ui: Cell::new(std::ptr::null_mut()),
            frame: Cell::new(0),
            ctx: RefCell::new(egui::Context::default()),
            last_response: RefCell::new(None),
            repaint: Cell::new(false),
            time: Cell::new(0.0),
        }
    }

    pub(crate) fn egui_ctx(&self) -> egui::Context {
        self.ctx.borrow().clone()
    }

    pub(crate) fn frame(&self) -> u64 {
        self.frame.get()
    }

    pub(crate) fn in_frame(&self) -> bool {
        !self.ui.get().is_null()
    }

    pub(crate) fn repaint_requested(&self) -> bool {
        self.repaint.get()
    }

    pub(crate) fn set_repaint_requested(&self, requested: bool) {
        self.repaint.set(requested);
    }

    pub(crate) fn advance_time(&self) -> f64 {
        let time = self.time.get();
        self.time.set(time + 1.0 / 60.0);
        time
    }

    pub(crate) fn set_response(&self, response: egui::Response) {
        *self.last_response.borrow_mut() = Some(response);
    }

    pub(crate) fn take_response(&self) -> Option<egui::Response> {
        self.last_response.borrow_mut().take()
    }

    /// Runs one frame procedure. The `Ui` pointer is installed for the call and
    /// restored afterwards, including when the procedure returns an error.
    pub(crate) fn run_proc(
        &self,
        ui: &mut egui::Ui,
        context: &EvalContext,
        proc: &Expr,
    ) -> EvalResult {
        self.frame.set(self.frame.get().wrapping_add(1));
        *self.last_response.borrow_mut() = None;
        self.enter(ui, || context.call(proc, &[]))
    }

    /// Points the slot at `ui` for the duration of `f`.
    pub(crate) fn enter<R>(&self, ui: &mut egui::Ui, f: impl FnOnce() -> R) -> R {
        let prev = self.ui.replace(ui);
        let _restore = Restore {
            slot: &self.ui,
            prev,
        };
        f()
    }

    /// Borrows the `Ui` installed by [`Self::enter`].
    ///
    /// The reference lasts only for `f`. Callers must not invoke Scheme while
    /// holding it; nested containers call [`Self::enter`] on the child `Ui` first.
    pub(crate) fn with_ui<R>(
        &self,
        proc_name: &str,
        f: impl FnOnce(&mut egui::Ui) -> Result<R, EvalError>,
    ) -> Result<R, EvalError> {
        let ptr = self.ui.get();
        if ptr.is_null() {
            return Err(outside(proc_name));
        }
        // SAFETY: `ptr` was stored by `enter` and is cleared when that call returns.
        // This reference does not outlive `f`. Nested Scheme runs only after `enter`
        // has pointed the slot at a different child `Ui`.
        f(unsafe { &mut *ptr })
    }
}

pub(crate) fn outside(proc_name: &str) -> EvalError {
    EvalError::new(
        ErrorKind::Other,
        format!("{proc_name}: called outside a frame"),
    )
}

struct Restore<'a> {
    slot: &'a Cell<*mut egui::Ui>,
    prev: *mut egui::Ui,
}

impl Drop for Restore<'_> {
    fn drop(&mut self) {
        self.slot.set(self.prev);
    }
}
