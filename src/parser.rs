mod conscell;

use crate::scanner::{ScanError, Scanner, Token};

pub enum ParseError {
    UnexpectedToken(Token),
    ScanError(ScanError),
    EndOfFile,
}

type ParseResult = Result<(), ParseError>;

pub struct Parser<Iter>
where
    Iter: Iterator<Item = char>,
{
    scanner: Scanner<Iter>,
}

impl<Iter> Parser<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            scanner: Scanner::new(iter),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        match self.scanner.get_token() {
            Ok(Token::LeftParan) => self.parse_expression(),
            Ok(token) => Err(ParseError::UnexpectedToken(token)),
            Err(ScanError::EndOfFile) => Err(ParseError::EndOfFile),
            Err(error) => Err(ParseError::ScanError(error)),
        }
    }

    fn parse_expression(&mut self) -> ParseResult {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new("(add 1 2)".chars());
        let _ = parser.parse();
    }
}
