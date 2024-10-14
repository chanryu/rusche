use std::fmt;

/// A location in the source code defined by a line and column number.
/// Be aware both line and column numbers are 0-based, even though they
/// need to be converted to 1-based when displayed to the user.
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
        // Convert 0-based line and column numbers to 1-based.
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

/// A region in the source code defined by a beginning and ending location. `Span` is used to
/// represent a range of token or expression in the source code.
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
            write!(f, "{}-{}", self.begin, self.end.column + 1) // (10:5, 10:7) => "11:6-8"
        } else {
            write!(f, "{}-{}", self.begin, self.end) // ((10:5, 11:3) => "11:6-12:4"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_display() {
        let span = Span::new(Loc::new(0, 1), Loc::new(0, 3));
        assert_eq!(format!("{}", span), "1:2-4");

        let span = Span::new(Loc::new(0, 1), Loc::new(2, 3));
        assert_eq!(format!("{}", span), "1:2-3:4");
    }
}
