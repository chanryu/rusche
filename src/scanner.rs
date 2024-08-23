use std::fmt;
use std::iter::{Iterator, Peekable};

const SYMBOL_DELIMITERS: &str = " \t\r\n()';\"";

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

pub struct TokenIter<Iter>
where
    Iter: Iterator<Item = char>,
{
    scanner: Scanner<Iter>,
    last_error: Option<ScanError>,
}

impl<Iter> TokenIter<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            scanner: Scanner::new(iter),
            last_error: None,
        }
    }

    pub fn get_last_error(&self) -> Option<&ScanError> {
        self.last_error.as_ref()
    }
}

impl<Iter> Iterator for TokenIter<Iter>
where
    Iter: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.scanner.get_token() {
            Ok(token) => {
                self.last_error = None;
                Some(token)
            }
            Err(error) => {
                self.last_error = Some(error);
                None
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ScanError {
    IncompleteString,
    InvalidNumber,
    EndOfFile,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::IncompleteString => write!(f, "Incomplete string"),
            ScanError::InvalidNumber => write!(f, "Invalid number"),
            ScanError::EndOfFile => write!(f, "EOF"),
        }
    }
}

pub type ScanResult = Result<Token, ScanError>;

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
            Some('(') => Ok(Token::OpenParen),
            Some(')') => Ok(Token::CloseParen),
            Some('\'') => Ok(Token::Quote),

            // string
            Some('"') => self.read_string(),

            // number
            Some(ch) if ch.is_ascii_digit() || ch == '.' => self.read_number(ch),

            // we allow all other characters to be a symbol
            Some(ch) => self.read_symbol(ch),

            None => Err(ScanError::EndOfFile),
        }
    }

    fn skip_spaces(&mut self) {
        while self.iter.next_if(|&ch| ch.is_whitespace()).is_some() {}
    }

    fn skip_comment(&mut self) -> bool {
        if self.iter.next_if_eq(&';').is_some() {
            self.iter.find(|&ch| is_newline_char(&ch));
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
                    text.push(ch);
                }
                ('"', false) => return Ok(Token::Str(text)),
                ('\\', false) => escaped = true,
                (ch, false) => text.push(ch),
            }
        }
        Err(ScanError::IncompleteString)
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
            .map(Token::Num)
            .map_err(|_| ScanError::InvalidNumber)
    }

    fn read_symbol(&mut self, first_char: char) -> ScanResult {
        let mut name = String::with_capacity(16);
        name.push(first_char);

        while let Some(ch) = self.iter.next_if(|&ch| !SYMBOL_DELIMITERS.contains(ch)) {
            name.push(ch);
        }

        Ok(Token::Sym(name))
    }
}

fn is_newline_char(ch: &char) -> bool {
    *ch == '\r' || *ch == '\n'
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Ok(Token::Str("valid string".to_string()))
        );
        parse_string_assert_eq!(
            r#""an escaped\" string""#,
            Ok(Token::Str(String::from("an escaped\" string")))
        );
        parse_string_assert_eq!(r#""incomplete string"#, Err(ScanError::IncompleteString));
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

        parse_number_assert_eq!("0", Ok(Token::Num(0_f64)));
        parse_number_assert_eq!("1", Ok(Token::Num(1_f64)));
        parse_number_assert_eq!("1.1", Ok(Token::Num(1.1)));
    }

    #[test]
    fn test_scanner_eof() {
        let mut scanner = Scanner::new("".chars());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());

        let mut scanner = Scanner::new("    ".chars());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());

        let mut scanner = Scanner::new("   ; comment".chars());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());

        let mut scanner = Scanner::new("".chars());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());
    }

    #[test]
    fn test_scanner_parans() {
        let mut scanner = Scanner::new("()(())(()())".chars());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Ok(Token::OpenParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Ok(Token::CloseParen), scanner.get_token());
        assert_eq!(Err(ScanError::EndOfFile), scanner.get_token());
    }

    #[test]
    fn test_scanner_all_tokens() {
        let all_tokens = r#"
        ; comment
        (add 1 2.34 (x y) "test" '(100 200 300))
        ; another comment"#;

        let mut scanner = Scanner::new(all_tokens.chars());
        assert_eq!(scanner.get_token(), Ok(Token::OpenParen));
        assert_eq!(scanner.get_token(), Ok(Token::Sym(String::from("add"))));
        assert_eq!(scanner.get_token(), Ok(Token::Num(1_f64)));
        assert_eq!(scanner.get_token(), Ok(Token::Num(2.34_f64)));
        assert_eq!(scanner.get_token(), Ok(Token::OpenParen));
        assert_eq!(scanner.get_token(), Ok(Token::Sym(String::from("x"))));
        assert_eq!(scanner.get_token(), Ok(Token::Sym(String::from("y"))));
        assert_eq!(scanner.get_token(), Ok(Token::CloseParen));
        assert_eq!(scanner.get_token(), Ok(Token::Str(String::from("test"))));
        assert_eq!(scanner.get_token(), Ok(Token::Quote));
        assert_eq!(scanner.get_token(), Ok(Token::OpenParen));
        assert_eq!(scanner.get_token(), Ok(Token::Num(100_f64)));
        assert_eq!(scanner.get_token(), Ok(Token::Num(200_f64)));
        assert_eq!(scanner.get_token(), Ok(Token::Num(300_f64)));
        assert_eq!(scanner.get_token(), Ok(Token::CloseParen));
        assert_eq!(scanner.get_token(), Ok(Token::CloseParen));
        assert_eq!(scanner.get_token(), Err(ScanError::EndOfFile));
    }
}
