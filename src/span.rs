use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// A location in the source code defined by a line and column number.
/// Be aware both line and column numbers are 0-based, even though they
/// need to be converted to 1-based when displayed to the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    pub line: usize,
    pub column: usize,
}

impl Loc {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn with_column_offset(&self, offset: usize) -> Loc {
        Self {
            line: self.line,
            column: self.column + offset,
        }
    }

    pub fn span_to(&self, end: Self) -> Span {
        Span::new(*self, end)
    }
}

pub(crate) fn intern_source(name: &str) -> &'static str {
    let mut guard = sources()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let sources = guard.get_or_insert_with(HashMap::new);
    if let Some((stored, _)) = sources.get_key_value(name) {
        return stored;
    }
    let stored: &'static str = Box::leak(Box::<str>::from(name));
    sources.insert(stored, ());
    stored
}

fn lookup_source(addr: NonZeroUsize) -> Option<&'static str> {
    let guard = sources()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.as_ref().and_then(|sources| {
        sources
            .keys()
            .find(|name| name.as_ptr() as usize == addr.get())
            .copied()
    })
}

fn sources() -> &'static Mutex<Option<HashMap<&'static str, ()>>> {
    static SOURCES: Mutex<Option<HashMap<&'static str, ()>>> = Mutex::new(None);
    &SOURCES
}

/// A region in the source code defined by a beginning and ending location. `Span` is used to
/// represent a range of token or expression in the source code.
///
/// An optional source name is stored as the address of an interned string
/// ([`NonZeroUsize::MIN`] means unnamed). That keeps the null niche free for
/// `Option<Span>`, so [`Expr`](crate::Expr) does not grow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub begin: Loc,
    pub end: Loc,
    source: NonZeroUsize,
}

impl Span {
    pub fn new(begin: Loc, end: Loc) -> Self {
        debug_assert!(begin < end, "invalid span: {begin:?}..{end:?}");

        Self {
            begin,
            end,
            source: NonZeroUsize::MIN,
        }
    }

    /// A span from `start.begin` to `end.end`, keeping `start`'s source name.
    pub fn between(start: Span, end: Span) -> Self {
        Self {
            begin: start.begin,
            end: end.end,
            source: start.source,
        }
    }

    /// File or buffer name, when this span was tagged with [`Span::with_source`].
    pub fn source(&self) -> Option<&'static str> {
        if self.source == NonZeroUsize::MIN {
            None
        } else {
            lookup_source(self.source)
        }
    }

    /// Tags this span with `name` (a file path, `"<eval>"`, `"<stdin>"`, …).
    ///
    /// The same name always yields the same interned string.
    pub fn with_source(self, name: &str) -> Self {
        self.with_interned_source(intern_source(name))
    }

    pub(crate) fn with_interned_source(mut self, name: &'static str) -> Self {
        let addr = name.as_ptr() as usize;
        self.source = NonZeroUsize::new(addr).expect("interned source is non-null");
        debug_assert!(
            self.source != NonZeroUsize::MIN,
            "interned source collided with the unnamed sentinel"
        );
        self
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source) = self.source() {
            write!(f, "{source}:")?;
        }
        if self.begin.line == self.end.line {
            if self.begin.column + 1 == self.end.column {
                // (10:5, 10:6) => "11:6"
                write!(f, "{}:{}", self.begin.line + 1, self.begin.column + 1)
            } else {
                // (10:5, 10:8) => "11:6-8"
                write!(
                    f,
                    "{}:{}-{}",
                    self.begin.line + 1,
                    self.begin.column + 1,
                    self.end.column
                )
            }
        } else {
            // ((10:5, 11:3) => "11:6-12:3"
            write!(
                f,
                "{}:{}-{}:{}",
                self.begin.line + 1,
                self.begin.column + 1,
                self.end.line + 1,
                self.end.column
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_display() {
        let span = Span::new(Loc::new(0, 1), Loc::new(0, 2));
        assert_eq!(format!("{}", span), "1:2");

        let span = Span::new(Loc::new(0, 1), Loc::new(0, 3));
        assert_eq!(format!("{}", span), "1:2-3");

        let span = Span::new(Loc::new(0, 9), Loc::new(2, 3));
        assert_eq!(format!("{}", span), "1:10-3:3");

        let span = Span::new(Loc::new(0, 1), Loc::new(0, 2)).with_source("lib.rsc");
        assert_eq!(span.source(), Some("lib.rsc"));
        assert_eq!(format!("{span}"), "lib.rsc:1:2");
        let wider = Span::between(span, Span::new(Loc::new(0, 1), Loc::new(0, 4)));
        assert_eq!(wider.source(), Some("lib.rsc"));
        assert_eq!(wider.end.column, 4);
    }
}
