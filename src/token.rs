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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_display() {
//         assert_eq!(format!("{}", Token::OpenParen), "(");
//         assert_eq!(format!("{}", Token::CloseParen), ")");
//         assert_eq!(format!("{}", Token::Quote), "'");
//         assert_eq!(format!("{}", Token::Quasiquote), "`");
//         assert_eq!(format!("{}", Token::Unquote), ",");
//         assert_eq!(format!("{}", Token::UnquoteSplicing), ",@");
//         assert_eq!(format!("{}", Token::Num(0.0)), "0");
//         assert_eq!(format!("{}", Token::Num(0.5)), "0.5");
//         assert_eq!(format!("{}", Token::Num(1.0)), "1");
//         assert_eq!(format!("{}", Token::Num(123.456)), "123.456");
//         assert_eq!(format!("{}", Token::Str("str".into())), "\"str\"");
//         assert_eq!(format!("{}", Token::Sym("sym".into())), "sym");
//     }
// }
