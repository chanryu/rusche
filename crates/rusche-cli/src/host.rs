use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static COMMAND_LINE: Mutex<Vec<String>> = Mutex::new(Vec::new());
static AT_COLUMN_ZERO: AtomicBool = AtomicBool::new(true);

pub fn set_command_line(args: Vec<String>) {
    *COMMAND_LINE.lock().unwrap() = args;
}

pub fn command_line() -> Vec<String> {
    COMMAND_LINE.lock().unwrap().clone()
}

pub fn at_column_zero() -> bool {
    AT_COLUMN_ZERO.load(Ordering::Relaxed)
}

pub fn set_at_column_zero(value: bool) {
    AT_COLUMN_ZERO.store(value, Ordering::Relaxed);
}

pub fn note_output(text: &str) {
    if text.is_empty() {
        return;
    }
    set_at_column_zero(text.ends_with('\n'));
}

/// If the last stdout write did not end in a newline, emit one so diagnostics start cleanly.
pub fn ensure_newline() {
    if !at_column_zero() {
        println!();
        set_at_column_zero(true);
    }
}
