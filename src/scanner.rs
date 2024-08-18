use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    Quote,
    Newline,
    Num(f64),
    Str(String),
    Sym(String),
}

#[derive(Debug, PartialEq)]
pub enum ScanError {
    IncompleteString,
    InvalidNumber,
    EndOfFile,
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
        self.skip_spaces();
        self.skip_comment();

        if let Some(ch) = self.iter.next() {
            match ch {
                '(' => Ok(Token::OpenParen),
                ')' => Ok(Token::CloseParen),
                '\'' => Ok(Token::Quote),

                // newline "\r" | "\r\n"
                '\r' => {
                    self.iter.next_if(|&ch| ch == '\n');
                    Ok(Token::Newline)
                }

                // newline "\n"
                '\n' => Ok(Token::Newline),

                // string
                '"' => self.read_string(),

                // number
                ch if ch.is_ascii_digit() => self.read_number(ch),

                // we allow all other characters to be a symbol
                ch => self.read_symbol(ch),
            }
        } else {
            Err(ScanError::EndOfFile)
        }
    }

    fn skip_spaces(&mut self) {
        while let Some(_) = self.iter.next_if(|&ch| ch == ' ' || ch == '\t') {}
    }

    fn skip_comment(&mut self) {
        if let Some(_) = self.iter.next_if_eq(&';') {
            while let Some(_) = self.iter.next_if(|ch| !is_newline_char(ch)) {}
        }
    }

    fn read_string(&mut self) -> ScanResult {
        let mut text = String::new();
        let mut escaped = false;
        while let Some(ch) = self.iter.next() {
            match ch {
                _ if escaped => {
                    escaped = false;
                    text.push(ch);
                }
                '\\' => escaped = true,
                '"' => return Ok(Token::Str(text)),
                _ => text.push(ch),
            }
        }
        Err(ScanError::IncompleteString)
    }

    fn read_number(&mut self, first_char: char) -> ScanResult {
        let mut has_decimal_point = false;
        let mut digits = first_char.to_string();
        while let Some(ch) = self
            .iter
            .next_if(|&ch| ch.is_ascii_digit() || (!has_decimal_point && ch == '.'))
        {
            digits.push(ch);
            has_decimal_point |= ch == '.';
        }
        if let Ok(value) = digits.parse::<f64>() {
            Ok(Token::Num(value))
        } else {
            Err(ScanError::InvalidNumber)
        }
    }

    fn read_symbol(&mut self, first_char: char) -> ScanResult {
        let mut name = first_char.to_string();
        loop {
            match self.iter.peek() {
                Some(&ch) if " \t\r\n()';\"".contains(ch) => break,
                Some(&ch) => {
                    name.push(ch);
                    self.iter.next();
                }
                None => break,
            }
        }
        Ok(Token::Sym(name))
    }
}

fn is_newline_char(ch: &char) -> bool {
    return *ch == '\r' || *ch == '\n';
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
                assert!($source.len() != 0);
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
        (add 1 2.34 (x y) "test" '(100 200 300))
        "#;

        let mut scanner = Scanner::new(all_tokens.chars());
        assert_eq!(scanner.get_token(), Ok(Token::Newline));
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
        assert_eq!(scanner.get_token(), Ok(Token::Newline));
        assert_eq!(scanner.get_token(), Err(ScanError::EndOfFile));
    }
}
