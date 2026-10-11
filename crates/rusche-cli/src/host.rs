use std::cell::RefCell;
use std::path::{Path, PathBuf};
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

thread_local! {
    static SOURCE_DIRS: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

/// Directory of the file currently being evaluated. Relative `load` paths join onto it.
pub struct SourceDirGuard;

impl Drop for SourceDirGuard {
    fn drop(&mut self) {
        SOURCE_DIRS.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// Treat `path` as the file being evaluated until the guard is dropped.
pub fn enter_source_file(path: &Path) -> SourceDirGuard {
    let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
    SOURCE_DIRS.with(|stack| stack.borrow_mut().push(dir));
    SourceDirGuard
}

pub fn current_source_dir() -> Option<PathBuf> {
    SOURCE_DIRS.with(|stack| stack.borrow().last().cloned())
}

/// If the last stdout write did not end in a newline, emit one so diagnostics start cleanly.
pub fn ensure_newline() {
    if !at_column_zero() {
        println!();
        set_at_column_zero(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_output_tracks_column_and_ignores_empty() {
        set_at_column_zero(true);
        note_output("");
        assert!(at_column_zero());

        note_output("hi");
        assert!(!at_column_zero());
        ensure_newline();
        assert!(at_column_zero());

        note_output("line\n");
        assert!(at_column_zero());

        set_command_line(vec!["rusche".into(), "a".into()]);
        assert_eq!(command_line(), vec!["rusche".to_string(), "a".to_string()]);
    }

    #[test]
    fn source_dir_stack_follows_the_file_being_evaluated() {
        assert!(current_source_dir().is_none());
        let outer = enter_source_file(Path::new("examples/fizzbuzz.rsc"));
        assert_eq!(current_source_dir().unwrap(), PathBuf::from("examples"));
        {
            let _inner = enter_source_file(Path::new("examples/lib/io.rsc"));
            assert_eq!(current_source_dir().unwrap(), PathBuf::from("examples/lib"));
        }
        assert_eq!(current_source_dir().unwrap(), PathBuf::from("examples"));
        drop(outer);
        assert!(current_source_dir().is_none());
    }
}
