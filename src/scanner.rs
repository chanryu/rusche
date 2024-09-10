use crate::token::Token;
use std::fmt;
use std::iter::{Iterator, Peekable};

const SYMBOL_DELIMITERS: &str = " \t\r\n()';\"";

#[derive(Debug, PartialEq)]
pub enum TokenError {
    IncompleteString,
    InvalidNumber,
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::IncompleteString => write!(f, "Incomplete string"),
            TokenError::InvalidNumber => write!(f, "Invalid number"),
        }
    }
}

type ScanResult = Result<Option<Token>, TokenError>;

pub struct Scanner<Iter>
where
    Iter: Iterator<Item = char>,
{
    iter: Peekable<Iter>,
}

impl<Iter> Scanner<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            iter: iter.peekable(),
        }
    }

    pub fn get_token(&mut self) -> ScanResult {
        loop {
            self.skip_spaces();
            if !self.skip_comment() {
                break;
            }
        }

        match self.iter.next() {
            Some('(') => Ok(Some(Token::OpenParen)),
            Some(')') => Ok(Some(Token::CloseParen)),

            Some('\'') => Ok(Some(Token::Quote)),
            Some('`') => Ok(Some(Token::Quasiquote)),
            Some(',') => {
                if self.iter.next_if_eq(&'@').is_some() {
                    Ok(Some(Token::UnquoteSplicing))
                } else {
                    Ok(Some(Token::Unquote))
                }
            }

            // string
            Some('"') => self.read_string(),

            // number
            Some(ch) if ch.is_ascii_digit() || ch == '.' => self.read_number(ch),

            // we allow all other characters to be a symbol
            Some(ch) => self.read_symbol(ch),

            None => Ok(None),
        }
    }

    fn skip_spaces(&mut self) {
        while self.iter.next_if(|&ch| ch.is_whitespace()).is_some() {}
    }

    fn skip_comment(&mut self) -> bool {
        if self.iter.next_if_eq(&';').is_some() {
            self.iter.find(|&ch| ch == '\r' || ch == '\n');
            true
        } else {
            false
        }
    }

    fn read_string(&mut self) -> ScanResult {
        let mut text = String::new();
        let mut escaped = false;
        for ch in &mut self.iter {
            match (ch, escaped) {
                (ch, true) => {
                    escaped = false;
                    match ch {
                        'n' => text.push('\n'),
                        'r' => text.push('\r'),
                        't' => text.push('\t'),
                        _ => text.push(ch),
                    }
                }
                ('"', false) => return Ok(Some(Token::Str(text))),
                ('\\', false) => escaped = true,
                (ch, false) => text.push(ch),
            }
        }
        Err(TokenError::IncompleteString)
    }

    fn read_number(&mut self, first_char: char) -> ScanResult {
        let mut has_decimal_point = first_char == '.';
        let mut digits = String::new();

        digits.push(first_char);
        while let Some(ch) = self
            .iter
            .next_if(|&ch| ch.is_ascii_digit() || (!has_decimal_point && ch == '.'))
        {
            digits.push(ch);
            if ch == '.' {
                has_decimal_point = true;
            }
        }

        digits
            .parse::<f64>()
            .map(|value| Some(Token::Num(value)))
            .map_err(|_| TokenError::InvalidNumber)
    }

    fn read_symbol(&mut self, first_char: char) -> ScanResult {
        let mut name = String::with_capacity(16);
        name.push(first_char);

        while let Some(ch) = self.iter.next_if(|&ch| !SYMBOL_DELIMITERS.contains(ch)) {
            name.push(ch);
        }

        Ok(Some(Token::Sym(name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num<T>(value: T) -> Token
    where
        T: Into<f64>,
    {
        Token::Num(value.into())
    }

    fn sym(name: &str) -> Token {
        Token::Sym(name.into())
    }

    fn ok_some<T, E>(t: T) -> Result<Option<T>, E> {
        Ok(Some(t))
    }

    #[test]
    fn test_read_string() {
        macro_rules! parse_string_assert_eq {
            ($source:literal, $expected:expr) => {
                let mut chars = $source.chars();
                assert_eq!(chars.next().unwrap(), '"');
                assert_eq!(Scanner::new(chars).read_string(), $expected);
            };
        }

        parse_string_assert_eq!(
            r#""valid string""#,
            Ok(Some(Token::Str("valid string".into())))
        );
        parse_string_assert_eq!(
            r#""an escaped\" string""#,
            Ok(Some(Token::Str(String::from("an escaped\" string"))))
        );
        parse_string_assert_eq!(r#""incomplete string"#, Err(TokenError::IncompleteString));
    }

    #[test]
    fn test_read_number() {
        macro_rules! parse_number_assert_eq {
            ($source:literal, $expected:expr) => {
                assert!(!$source.is_empty());
                let mut chars = $source.chars();
                let first_char = chars.next().unwrap();
                assert_eq!(Scanner::new(chars).read_number(first_char), $expected);
            };
        }

        parse_number_assert_eq!("0", ok_some(num(0)));
        parse_number_assert_eq!("1", ok_some(num(1)));
        parse_number_assert_eq!("1.1", ok_some(num(1.1)));
    }

    #[test]
    fn test_scanner_eof() {
        let mut scanner = Scanner::new("".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("    ".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("   ; comment".chars());
        assert_eq!(scanner.get_token(), Ok(None));

        let mut scanner = Scanner::new("".chars());
        assert_eq!(scanner.get_token(), Ok(None));
        assert_eq!(scanner.get_token(), Ok(None));
        assert_eq!(scanner.get_token(), Ok(None));
    }

    #[test]
    fn test_scanner_parans() {
        let mut scanner = Scanner::new("()(())(()())".chars());
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), Ok(None));
    }

    #[test]
    fn test_scanner_all_tokens() {
        let all_tokens = r#"
        ; comment
        (add 1 2.34 (x y) "test" '(100 200 300))
        ; another comment"#;

        let mut scanner = Scanner::new(all_tokens.chars());
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(sym("add")));
        assert_eq!(scanner.get_token(), ok_some(num(1)));
        assert_eq!(scanner.get_token(), ok_some(num(2.34)));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(sym("x")));
        assert_eq!(scanner.get_token(), ok_some(sym("y")));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::Str("test".into())));
        assert_eq!(scanner.get_token(), ok_some(Token::Quote));
        assert_eq!(scanner.get_token(), ok_some(Token::OpenParen));
        assert_eq!(scanner.get_token(), ok_some(num(100)));
        assert_eq!(scanner.get_token(), ok_some(num(200)));
        assert_eq!(scanner.get_token(), ok_some(num(300)));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), ok_some(Token::CloseParen));
        assert_eq!(scanner.get_token(), Ok(None));
    }
}
