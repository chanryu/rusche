use std::fmt::{Display, Formatter, Result as FmtResult};

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

impl Display for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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

#[derive(Debug, Clone)]
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
            | Token::Quasiquote(loc)
            | Token::Unquote(loc) => Span::new(*loc, 1),
            Token::UnquoteSplicing(loc) => Span::new(*loc, 2),
            Token::Num(span, _) | Token::Str(span, _) | Token::Sym(span, _) => *span,
        }
    }

    pub fn loc(&self) -> Loc {
        match self {
            Token::OpenParen(loc)
            | Token::CloseParen(loc)
            | Token::Quote(loc)
            | Token::Quasiquote(loc)
            | Token::Unquote(loc)
            | Token::UnquoteSplicing(loc)
            | Token::Num(Span { loc, .. }, _)
            | Token::Str(Span { loc, .. }, _)
            | Token::Sym(Span { loc, .. }, _) => *loc,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::OpenParen(_), Token::OpenParen(_)) => true,
            (Token::CloseParen(_), Token::CloseParen(_)) => true,
            (Token::Quote(_), Token::Quote(_)) => true,
            (Token::Quasiquote(_), Token::Quasiquote(_)) => true,
            (Token::Unquote(_), Token::Unquote(_)) => true,
            (Token::UnquoteSplicing(_), Token::UnquoteSplicing(_)) => true,
            (Token::Num(_, a), Token::Num(_, b)) => a == b,
            (Token::Str(_, a), Token::Str(_, b)) => a == b,
            (Token::Sym(_, a), Token::Sym(_, b)) => a == b,
            _ => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
    fn test_loc() {
        let loc = Loc::new(99, 999);
        let expected_loc = loc;
        assert_eq!(expected_loc, Token::OpenParen(loc).loc());
        assert_eq!(expected_loc, Token::CloseParen(loc).loc());
        assert_eq!(expected_loc, Token::Quote(loc).loc());
        assert_eq!(expected_loc, Token::Quasiquote(loc).loc());
        assert_eq!(expected_loc, Token::Unquote(loc).loc());
        assert_eq!(expected_loc, Token::UnquoteSplicing(loc).loc());
        assert_eq!(expected_loc, Token::Num(Span::new(loc, 1), 0.0).loc());
        assert_eq!(
            expected_loc,
            Token::Str(Span::new(loc, 5), "str".to_string()).loc()
        );
        assert_eq!(
            expected_loc,
            Token::Sym(Span::new(loc, 3), "sym".to_string()).loc()
        );
    }

    #[test]
    fn test_span_len_1() {
        macro_rules! assert_token_span_length_eq {
            ($length:literal, $token_case:ident) => {
                assert_eq!($length, Token::$token_case(Loc::new(0, 0)).span().len);
            };
        }
        assert_token_span_length_eq!(1, OpenParen);
        assert_token_span_length_eq!(1, CloseParen);
        assert_token_span_length_eq!(1, Quote);
        assert_token_span_length_eq!(1, Quasiquote);
        assert_token_span_length_eq!(1, Unquote);
        assert_token_span_length_eq!(2, UnquoteSplicing);
    }

    #[test]
    fn test_display() {
        macro_rules! assert_token_format_eq {
            ($token_case:ident, $formatted:literal) => {
                assert_eq!(
                    format!("{}", Token::$token_case(Loc::new(1, 1))),
                    $formatted
                );
            };
            ($token_case:ident($value:expr), $formatted:literal) => {
                assert_eq!(
                    format!(
                        "{}",
                        Token::$token_case(Span::new(Loc::new(1, 1), 1), $value)
                    ),
                    $formatted
                );
            };
        }
        assert_token_format_eq!(CloseParen, ")");
        assert_token_format_eq!(Quote, "'");
        assert_token_format_eq!(Quasiquote, "`");
        assert_token_format_eq!(Unquote, ",");
        assert_token_format_eq!(UnquoteSplicing, ",@");
        assert_token_format_eq!(Num(0.0), "0");
        assert_token_format_eq!(Num(0.5), "0.5");
        assert_token_format_eq!(Num(1.0), "1");
        assert_token_format_eq!(Num(123.456), "123.456");
        assert_token_format_eq!(Num(123.456), "123.456");
        assert_token_format_eq!(Str("str".to_string()), "\"str\"");
        assert_token_format_eq!(Sym("sym".to_string()), "sym");
    }
}
