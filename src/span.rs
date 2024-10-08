use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn span_from(&self, begin: Self) -> Span {
        Span::new(begin, *self)
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub begin: Loc,
    pub end: Loc,
}

impl Span {
    pub fn new(begin: Loc, end: Loc) -> Self {
        Self { begin, end }
    }

    pub fn len(&self) -> usize {
        self.end.column - self.begin.column
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.begin.line == self.end.line {
            write!(f, "{}-{}", self.begin, self.end.column)
        } else {
            write!(f, "{}-{}", self.begin, self.end)
        }
    }
}
