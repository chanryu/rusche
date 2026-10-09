use std::cell::{Cell, RefCell};

thread_local! {
    static COMMAND_LINE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static AT_COLUMN_ZERO: Cell<bool> = const { Cell::new(true) };
}

pub fn set_command_line(args: Vec<String>) {
    COMMAND_LINE.with(|cell| *cell.borrow_mut() = args);
}

pub fn command_line() -> Vec<String> {
    COMMAND_LINE.with(|cell| cell.borrow().clone())
}

pub fn at_column_zero() -> bool {
    AT_COLUMN_ZERO.with(|cell| cell.get())
}

pub fn set_at_column_zero(value: bool) {
    AT_COLUMN_ZERO.with(|cell| cell.set(value));
}

pub fn note_output(text: &str) {
    if text.is_empty() {
        return;
    }
    set_at_column_zero(text.ends_with('\n'));
}
