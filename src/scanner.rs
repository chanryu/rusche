use std::iter::{Iterator, Peekable};

#[derive(Debug)]
pub enum Token {
    LeftParan,
    RightParan,
    Quote,
    Backtick,
    Newline,
    Number(f64),
    String(String),
    Symbol(String),
    Comment(String),
}

#[derive(Debug)]
pub enum ScanError {
    IncompleteString,
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

            Some(';') => Ok(self.read_comment()),

            // string
            Some('"') => self.try_read_string(),

            // number
            Some(ch) if ch.is_ascii_digit() || ch == '.' => Ok(self.read_number(ch)),

            // we allow all other characters to be a symbol
            Some(ch) => Ok(self.read_symbol(ch)),

            None => Err(ScanError::EndOfFile),
        }
    }

    fn skip_spaces(&mut self) {
        while let Some(_) = self.iter.next_if(|&ch| ch == ' ' || ch == '\t') {}
    }

    fn read_comment(&mut self) -> Token {
        let mut text = String::new();
        while let Some(ch) = self.iter.next_if(|&ch| ch != '\r' || ch != '\n') {
            text.push(ch);
        }
        Token::Comment(text)
    }

    fn read_number(&mut self, first_char: char) -> Token {
        let mut has_decimal_point = first_char == '.';
        let mut digits = first_char.to_string();
        while let Some(ch) = self
            .iter
            .next_if(|&ch| ch.is_ascii_digit() || (!has_decimal_point && ch == '.'))
        {
            digits.push(ch);
            has_decimal_point |= ch == '.';
        }
        Token::Number(digits.parse::<f64>().unwrap())
    }

    fn read_symbol(&mut self, first_char: char) -> Token {
        let mut name = first_char.to_string();
        loop {
            match self.iter.peek() {
                Some(ch) if " \t\r\n()';\"".contains(*ch) => break,
                Some(ch) => {
                    name.push(*ch);
                    self.iter.next();
                }
                None => break,
            }
        }
        Token::Symbol(name)
    }

    fn try_read_string(&mut self) -> ScanResult {
        let mut text = String::new();
        loop {
            match self.iter.next() {
                Some('"') => return Ok(Token::String(text)),
                Some(ch) => text.push(ch),
                None => return Err(ScanError::IncompleteString),
            }
        }
    }
}
