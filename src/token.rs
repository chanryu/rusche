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

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn num<T>(value: T) -> Token
    where
        T: Into<f64>,
    {
        Token::Num(value.into())
    }

    pub fn sym(name: &str) -> Token {
        Token::Sym(name.to_string())
    }

    pub fn str(name: &str) -> Token {
        Token::Str(name.to_string())
    }
}
