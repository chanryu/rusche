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

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenParen(Loc),
    CloseParen(Loc),
    Quote(Loc),
    Quasiquote(Loc),
    Unquote(Loc),
    UnquoteSplicing(Loc),
    Num(Loc, f64),
    Str(Loc, String),
    Sym(Loc, String),
}

impl Token {
    pub fn loc(&self) -> Loc {
        match self {
            Token::OpenParen(loc)
            | Token::CloseParen(loc)
            | Token::Quote(loc)
            | Token::Quasiquote(loc)
            | Token::Unquote(loc)
            | Token::UnquoteSplicing(loc)
            | Token::Num(loc, _)
            | Token::Str(loc, _)
            | Token::Sym(loc, _) => *loc,
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
        let loc = Loc::new(1, 0);
        assert_eq!(format!("{}", Token::OpenParen(loc)), "(");
        assert_eq!(format!("{}", Token::CloseParen(loc)), ")");
        assert_eq!(format!("{}", Token::Quote(loc)), "'");
        assert_eq!(format!("{}", Token::Quasiquote(loc)), "`");
        assert_eq!(format!("{}", Token::Unquote(loc)), ",");
        assert_eq!(format!("{}", Token::UnquoteSplicing(loc)), ",@");
        assert_eq!(format!("{}", Token::Num(loc, 0.0)), "0");
        assert_eq!(format!("{}", Token::Num(loc, 0.5)), "0.5");
        assert_eq!(format!("{}", Token::Num(loc, 1.0)), "1");
        assert_eq!(format!("{}", Token::Num(loc, 123.456)), "123.456");
        assert_eq!(format!("{}", Token::Str(loc, "str".into())), "\"str\"");
        assert_eq!(format!("{}", Token::Sym(loc, "sym".into())), "sym");
    }
}
