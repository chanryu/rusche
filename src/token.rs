use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Quote,
    Num(f64),
    Str(String),
    Sym(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Quote => write!(f, "'"),
            Token::Num(value) => write!(f, "{}", value),
            Token::Str(text) => write!(f, "\"{}\"", text),
            Token::Sym(name) => write!(f, "{}", name),
        }
    }
}
