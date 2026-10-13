//! Mutable string foreign, because `TextEdit` needs an `&mut String` across frames.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

use rusche::{
    arity_error, get_exact_1_arg, get_exact_2_args, ErrorKind, EvalContext, EvalError, EvalResult,
    Expr, List, NIL,
};

use crate::host::Host;
use crate::value::{self, eval_args};

/// Script-owned text for [`text-edit`](crate).
///
/// Rusche strings are immutable. A buffer is the value `text-edit` mutates, and
/// the value the host reads back with [`Buffer::text`].
pub struct Buffer {
    text: RefCell<String>,
    cursor_end: Cell<bool>,
}

impl Buffer {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: RefCell::new(text.into()),
            cursor_end: Cell::new(false),
        }
    }

    pub fn text(&self) -> String {
        self.text.borrow().clone()
    }

    pub fn set_text(&self, text: impl Into<String>) {
        *self.text.borrow_mut() = text.into();
    }

    pub(crate) fn request_cursor_end(&self) {
        self.cursor_end.set(true);
    }

    pub(crate) fn take_cursor_end(&self) -> bool {
        self.cursor_end.replace(false)
    }

    /// Moves the text out so `TextEdit` can borrow it without holding this `RefCell`.
    /// Dropping the guard writes the text back, including after a panic.
    pub(crate) fn take_text(&self) -> TextGuard<'_> {
        TextGuard {
            slot: &self.text,
            text: std::mem::take(&mut *self.text.borrow_mut()),
        }
    }
}

pub(crate) struct TextGuard<'a> {
    slot: &'a RefCell<String>,
    pub(crate) text: String,
}

impl Drop for TextGuard<'_> {
    fn drop(&mut self) {
        *self.slot.borrow_mut() = std::mem::take(&mut self.text);
    }
}

impl rusche::ForeignObject for Buffer {
    fn type_name(&self) -> &'static str {
        "buffer"
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<buffer {} chars>", self.text.borrow().chars().count())
    }
}

pub(crate) fn require_buffer(proc_name: &str, value: &Expr) -> Result<Rc<Buffer>, EvalError> {
    value
        .as_foreign::<Buffer>()
        .ok_or_else(|| value::type_error(proc_name, value, "a buffer"))
}

pub(crate) fn buffer(
    _host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    let text = match values.len() {
        0 => String::new(),
        1 => value::require_str(proc_name, &values[0])?.to_string(),
        n => return Err(arity_error(proc_name, 0..=1, n)),
    };
    Ok(Expr::foreign(Buffer::new(text)))
}

pub(crate) fn buffer_text(
    _host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let value = rusche::eval(arg, context)?;
    let buffer = require_buffer(proc_name, &value)?;
    Ok(Expr::from(buffer.text()))
}

pub(crate) fn buffer_set(
    _host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (buffer_expr, text_expr) = get_exact_2_args(proc_name, args)?;
    let buffer_value = rusche::eval(buffer_expr, context)?;
    let text = rusche::eval_into_str(proc_name, text_expr, context)?;
    let buffer = require_buffer(proc_name, &buffer_value)?;
    buffer.set_text(text);
    Ok(NIL)
}

pub(crate) fn buffer_cursor(
    _host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (buffer_expr, place_expr) = get_exact_2_args(proc_name, args)?;
    let buffer_value = rusche::eval(buffer_expr, context)?;
    let place_value = rusche::eval(place_expr, context)?;
    let place = value::require_sym(proc_name, &place_value)?;
    if place != "end" {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: expected 'end, got `{place_value}`"),
        ));
    }
    let buffer = require_buffer(proc_name, &buffer_value)?;
    buffer.request_cursor_end();
    Ok(NIL)
}
