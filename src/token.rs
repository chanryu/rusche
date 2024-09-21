use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Loc {
    pub line: usize,
    pub column: usize,
}

impl Loc {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Span {
    pub loc: Loc,
    pub len: usize,
}

impl Span {
    pub fn new(loc: Loc, len: usize) -> Self {
        Self { loc, len }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenParen(Loc),
    CloseParen(Loc),
    Quote(Loc),
    Quasiquote(Loc),
    Unquote(Loc),
    UnquoteSplicing(Loc),
    Num(Span, f64),
    Str(Span, String),
    Sym(Span, String),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::OpenParen(loc)
            | Token::CloseParen(loc)
            | Token::Quote(loc)
            | Token::Unquote(loc) => Span::new(*loc, 1),
            Token::Quasiquote(loc) | Token::UnquoteSplicing(loc) => Span::new(*loc, 2),
            Token::Num(span, _) | Token::Str(span, _) | Token::Sym(span, _) => *span,
        }
    }

    pub fn loc(&self) -> &Loc {
        match self {
            Token::OpenParen(loc)
            | Token::CloseParen(loc)
            | Token::Quote(loc)
            | Token::Quasiquote(loc)
            | Token::Unquote(loc)
            | Token::UnquoteSplicing(loc)
            | Token::Num(Span { loc, .. }, _)
            | Token::Str(Span { loc, .. }, _)
            | Token::Sym(Span { loc, .. }, _) => loc,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::OpenParen(_) => write!(f, "("),
            Token::CloseParen(_) => write!(f, ")"),
            Token::Quote(_) => write!(f, "'"),
            Token::Quasiquote(_) => write!(f, "`"),
            Token::Unquote(_) => write!(f, ","),
            Token::UnquoteSplicing(_) => write!(f, ",@"),
            Token::Num(_, value) => write!(f, "{}", value),
            Token::Str(_, text) => write!(f, "\"{}\"", text),
            Token::Sym(_, name) => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let loc = Loc::new(1, 1);
        assert_eq!(format!("{}", Token::OpenParen(loc)), "(");
        assert_eq!(format!("{}", Token::CloseParen(loc)), ")");
        assert_eq!(format!("{}", Token::Quote(loc)), "'");
        assert_eq!(format!("{}", Token::Quasiquote(loc)), "`");
        assert_eq!(format!("{}", Token::Unquote(loc)), ",");
        assert_eq!(format!("{}", Token::UnquoteSplicing(loc)), ",@");
        assert_eq!(format!("{}", Token::Num(Span::new(loc, 1), 0.0)), "0");
        assert_eq!(format!("{}", Token::Num(Span::new(loc, 3), 0.5)), "0.5");
        assert_eq!(format!("{}", Token::Num(Span::new(loc, 1), 1.0)), "1");
        assert_eq!(
            format!("{}", Token::Num(Span::new(loc, 7), 123.456)),
            "123.456"
        );
        assert_eq!(
            format!("{}", Token::Str(Span::new(loc, 5), "str".into())),
            "\"str\""
        );
        assert_eq!(
            format!("{}", Token::Sym(Span::new(loc, 3), "sym".into())),
            "sym"
        );
    }

    #[test]
    fn test_loc() {
        let loc = Loc::new(1, 1);
        assert_eq!(*Token::OpenParen(loc).loc(), loc);
        assert_eq!(*Token::CloseParen(loc).loc(), loc);
        assert_eq!(*Token::Quote(loc).loc(), loc);
        assert_eq!(*Token::Quasiquote(loc).loc(), loc);
        assert_eq!(*Token::Unquote(loc).loc(), loc);
        assert_eq!(*Token::UnquoteSplicing(loc).loc(), loc);
        assert_eq!(*Token::Num(Span::new(loc, 1), 0.0).loc(), loc);
        assert_eq!(*Token::Str(Span::new(loc, 5), "str".into()).loc(), loc);
        assert_eq!(*Token::Sym(Span::new(loc, 3), "sym".into()).loc(), loc);
    }
}
