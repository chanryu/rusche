use std::iter::{Iterator, Peekable};

#[derive(Debug)]
pub enum Token {
    LeftParan,
    RightParan,
    Quote,
    Backtick,
    Newline,
    // Number(f64),
    String(String),
    Symbol(String),
}

#[derive(Debug)]
pub enum ScanError {
    UnexpectedChar(char),
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
        // skip any spaces and/or comment
        while let Some(ch) = self.iter.next_if(|&ch| ch.is_ascii_whitespace()) {
            if ch == ';' {
                while let Some(_) = self.iter.next_if(|&ch| ch != '\r' || ch != '\n') {}
                break;
            }
        }

        match self.iter.next() {
            // newline "\r" | "\r\n"
            Some('\r') => {
                self.iter.next_if(|&ch| ch == '\n');
                Ok(Token::Newline)
            }
            // newline "\n"
            Some('\n') => Ok(Token::Newline),

            Some('(') => Ok(Token::LeftParan),
            Some(')') => Ok(Token::RightParan),
            Some('\'') => Ok(Token::Quote),
            Some('`') => Ok(Token::Backtick),

            // quoted string
            Some('"') => self.read_string(),

            Some(ch) if !ch.is_control() => self.read_symbol(ch),

            Some(ch) => Err(ScanError::UnexpectedChar(ch)),
            None => Err(ScanError::EndOfFile),
        }
    }

    fn read_symbol(&mut self, first_char: char) -> ScanResult {
        let mut name = first_char.to_string();
        loop {
            match self.iter.peek() {
                Some(ch) if "()'`".contains(*ch) || ch.is_ascii_whitespace() => break,
                Some(ch) => {
                    name.push(*ch);
                    self.iter.next();
                }
                None => break,
            }
        }
        Ok(Token::Symbol(name))
    }

    fn read_string(&mut self) -> ScanResult {
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
