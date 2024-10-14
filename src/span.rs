use std::fmt;

/// A location in the source code defined by a line and column number.
/// Be aware both line and column numbers are 0-based.
/// However, when displayed to the user, they should be converted to 1-based.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

/// A region in the source code defined by a beginning and ending location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub begin: Loc,
    pub end: Loc,
}

impl Span {
    pub fn new(begin: Loc, end: Loc) -> Self {
        debug_assert!(
            begin.line < end.line || (begin.line == end.line && begin.column < end.column)
        );

        Self { begin, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.begin.line == self.end.line {
            write!(f, "{}-{}", self.begin, self.end.column) // 10:5, 10:7 => "10:5-7"
        } else {
            write!(f, "{}-{}", self.begin, self.end) // 10:5, 11:3 => "10:5-11:3"
        }
    }
}
