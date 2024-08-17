use std::iter::{Iterator, Peekable};

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParan,
    RightParan,
    Quote,
    Backtick,
    Newline,
    Number(f64),
    String(String),
    Symbol(String),
}

#[derive(Debug, PartialEq)]
pub enum ScanError {
    IncompleteString,
    InvalidNumber,
    EndOfFile,
}

type ScanResult = Result<Token, ScanError>;

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

        match self.iter.next() {
            Some('(') => Ok(Token::LeftParan),
            Some(')') => Ok(Token::RightParan),
            Some('\'') => Ok(Token::Quote),
            Some('`') => Ok(Token::Backtick),

            // newline "\r" | "\r\n"
            Some('\r') => {
                self.iter.next_if(|&ch| ch == '\n');
                Ok(Token::Newline)
            }

            // newline "\n"
            Some('\n') => Ok(Token::Newline),

            // string
            Some('"') => self.read_string(),

            // number
            Some(ch) if ch.is_ascii_digit() => self.read_number(ch),

            // we allow all other characters to be a symbol
            Some(ch) => self.read_symbol(ch),

            None => Err(ScanError::EndOfFile),
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
        loop {
            match self.iter.next() {
                Some(ch) if escaped => {
                    escaped = false;
                    text.push(ch);
                }
                Some('\\') => escaped = true,
                Some('"') => return Ok(Token::String(text)),
                Some(ch) => text.push(ch),
                None => return Err(ScanError::IncompleteString),
            }
        }
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
        match digits.parse::<f64>() {
            Ok(value) => Ok(Token::Number(value)),
            Err(_) => Err(ScanError::InvalidNumber),
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
        Ok(Token::Symbol(name))
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
            Ok(Token::String("valid string".to_string()))
        );
        parse_string_assert_eq!(
            r#""an escaped\" string""#,
            Ok(Token::String(String::from("an escaped\" string")))
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

        parse_number_assert_eq!("0", Ok(Token::Number(0_f64)));
        parse_number_assert_eq!("1", Ok(Token::Number(1_f64)));
        parse_number_assert_eq!("1.1", Ok(Token::Number(1.1)));
    }
}
