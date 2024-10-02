use std::fmt::{Display, Formatter, Result};

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

impl Display for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
