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
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Token::OpenParen), "(");
        assert_eq!(format!("{}", Token::CloseParen), ")");
        assert_eq!(format!("{}", Token::Quote), "'");
        assert_eq!(format!("{}", Token::Num(0.0)), "0");
        assert_eq!(format!("{}", Token::Num(0.5)), "0.5");
        assert_eq!(format!("{}", Token::Num(1.0)), "1");
        assert_eq!(format!("{}", Token::Num(123.456)), "123.456");
        assert_eq!(format!("{}", Token::Str("str".into())), "\"str\"");
        assert_eq!(format!("{}", Token::Sym("sym".into())), "sym");
    }
}
