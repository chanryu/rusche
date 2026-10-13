use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// State shared by natives, the REPL, and diagnostics.
///
/// Natives that need it capture an `Rc<Host>`. Column tracking and the `load`
/// directory stack use interior mutability, so callers share the `Rc` directly.
pub struct Host {
    command_line: Vec<String>,
    at_column_zero: Cell<bool>,
    source_dirs: RefCell<Vec<PathBuf>>,
    /// File text retained so a diagnostic can show a file other than the one
    /// the top-level printer was given. Entries stay for the process lifetime.
    source_texts: RefCell<Vec<(String, String)>>,
}

impl Host {
    pub fn new(command_line: Vec<String>) -> Rc<Self> {
        Rc::new(Self {
            command_line,
            at_column_zero: Cell::new(true),
            source_dirs: RefCell::new(Vec::new()),
            source_texts: RefCell::new(Vec::new()),
        })
    }

    pub fn command_line(&self) -> &[String] {
        &self.command_line
    }

    pub fn at_column_zero(&self) -> bool {
        self.at_column_zero.get()
    }

    pub fn set_at_column_zero(&self, value: bool) {
        self.at_column_zero.set(value);
    }

    pub fn note_output(&self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.set_at_column_zero(text.ends_with('\n'));
    }

    /// Treat `path` as the file being evaluated until the guard is dropped.
    /// Relative `load` paths join onto its directory.
    pub fn enter_source_file(self: &Rc<Self>, path: &Path) -> SourceDirGuard {
        let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
        self.source_dirs.borrow_mut().push(dir);
        SourceDirGuard {
            host: Rc::clone(self),
        }
    }

    pub fn current_source_dir(&self) -> Option<PathBuf> {
        self.source_dirs.borrow().last().cloned()
    }

    /// Remembers `text` under `name` so diagnostics can print it after `load` returns.
    ///
    /// A later load of the same name replaces the text.
    pub fn remember_source(&self, name: &str, text: &str) {
        let mut texts = self.source_texts.borrow_mut();
        if let Some(slot) = texts.iter_mut().find(|(stored, _)| stored == name) {
            slot.1.clear();
            slot.1.push_str(text);
        } else {
            texts.push((name.to_string(), text.to_string()));
        }
    }

    /// Text previously stored with [`Host::remember_source`].
    pub fn source_text(&self, name: &str) -> Option<String> {
        self.source_texts
            .borrow()
            .iter()
            .rev()
            .find(|(stored, _)| stored == name)
            .map(|(_, text)| text.clone())
    }

    /// If the last stdout write did not end in a newline, emit one so diagnostics start cleanly.
    pub fn ensure_newline(&self) {
        if !self.at_column_zero() {
            println!();
            self.set_at_column_zero(true);
        }
    }
}

/// Pops the source-directory stack when dropped.
pub struct SourceDirGuard {
    host: Rc<Host>,
}

impl Drop for SourceDirGuard {
    fn drop(&mut self) {
        self.host.source_dirs.borrow_mut().pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_output_tracks_column_and_ignores_empty() {
        let host = Host::new(vec!["rusche".into(), "a".into()]);
        assert!(host.at_column_zero());
        host.note_output("");
        assert!(host.at_column_zero());

        host.note_output("hi");
        assert!(!host.at_column_zero());
        host.ensure_newline();
        assert!(host.at_column_zero());

        host.note_output("line\n");
        assert!(host.at_column_zero());

        assert_eq!(host.command_line(), ["rusche".to_string(), "a".to_string()]);
    }

    #[test]
    fn source_dir_stack_follows_the_file_being_evaluated() {
        let host = Host::new(Vec::new());
        assert!(host.current_source_dir().is_none());
        let outer = host.enter_source_file(Path::new("examples/fizzbuzz.rsc"));
        assert_eq!(
            host.current_source_dir().unwrap(),
            PathBuf::from("examples")
        );
        {
            let _inner = host.enter_source_file(Path::new("examples/lib/io.rsc"));
            assert_eq!(
                host.current_source_dir().unwrap(),
                PathBuf::from("examples/lib")
            );
        }
        assert_eq!(
            host.current_source_dir().unwrap(),
            PathBuf::from("examples")
        );
        drop(outer);
        assert!(host.current_source_dir().is_none());
    }
}
