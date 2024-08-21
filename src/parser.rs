use crate::expr::{Cons, Expr, NIL};
use crate::scanner::{ScanError, Scanner, Token};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    ScanError(ScanError),
}

type ParseResult = Result<Expr, ParseError>;

struct ParseContext {
    list_began: bool,
    car: Option<Expr>,
}

pub struct Parser<Iter>
where
    Iter: Iterator<Item = char>,
{
    scanner: Scanner<Iter>,
    contexts: Vec<ParseContext>,
}

impl<Iter> Parser<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(iter: Iter) -> Self {
        Self {
            scanner: Scanner::new(iter),
            contexts: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        loop {
            let expr = match self.scanner.get_token() {
                Ok(Token::OpenParen) => {
                    self.begin_list();
                    continue;
                }
                Ok(Token::CloseParen) => self.end_list()?,
                Ok(Token::Sym(text)) => Expr::Sym(text),
                Ok(Token::Str(text)) => Expr::Str(text),
                Ok(Token::Num(value)) => Expr::Num(value),
                Ok(token) => return Err(ParseError::UnexpectedToken(token)),
                Err(error) => return Err(ParseError::ScanError(error)),
            };

            if let Some(context) = self.contexts.last_mut() {
                if context.car.is_none() {
                    context.car = Some(expr);
                } else {
                    self.contexts.push(ParseContext {
                        list_began: false,
                        car: Some(expr),
                    });
                }
            } else {
                return Ok(expr);
            }
        }
    }

    fn begin_list(&mut self) {
        self.contexts.push(ParseContext {
            list_began: true,
            car: None,
        })
    }

    fn end_list(&mut self) -> ParseResult {
        let mut expr = NIL;
        while let Some(context) = self.contexts.pop() {
            if let Some(car) = context.car {
                expr = Expr::List(Some(Cons::new(car, expr)));
            } else {
                assert!(expr == NIL);
            }
            if context.list_began {
                return Ok(expr);
            }
        }
        Err(ParseError::UnexpectedToken(Token::CloseParen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(value: i32) -> Expr {
        Expr::Num(f64::from(value))
    }

    fn sym(text: &str) -> Expr {
        Expr::Sym(String::from(text))
    }

    fn list(car: Expr, cdr: Expr) -> Expr {
        Expr::List(Some(Cons::new(car, cdr)))
    }

    #[test]
    fn test_parser() {
        let mut parser = Parser::new("(add 1 2)".chars());
        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list(sym("add"), list(num(1), list(num(2), NIL)));
        assert_eq!(parsed_expr, expected_expr);
    }
}
