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
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub loc: Loc,
    pub end: Loc,
}

impl Span {
    pub fn new(loc: Loc, end: Loc) -> Self {
        Self { loc, end }
    }

    pub fn len(&self) -> usize {
        self.end.column - self.loc.column
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.loc.line == self.end.line {
            write!(f, "{}-{}", self.loc, self.end.column)
        } else {
            write!(f, "{}-{}", self.loc, self.end)
        }
    }
}
