use crate::expr::{Cons, Expr};
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
            let mut expr = Expr::Nil;

            match self.scanner.get_token() {
                Ok(Token::OpenParen) => {
                    self.begin_list();
                    continue;
                }
                Ok(Token::CloseParen) => expr = self.end_list()?,
                Ok(Token::Sym(text)) => expr = Expr::Sym(text),
                Ok(Token::Str(text)) => expr = Expr::Str(text),
                Ok(Token::Num(value)) => expr = Expr::Num(value),
                Ok(token) => return Err(ParseError::UnexpectedToken(token)),
                Err(error) => return Err(ParseError::ScanError(error)),
            }

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
        let mut expr = Expr::Nil;
        loop {
            if let Some(context) = self.contexts.pop() {
                if let Some(car) = context.car {
                    expr = Expr::List(Box::new(Cons { car, cdr: expr }));
                } else {
                    assert!(expr == Expr::Nil);
                }
                if context.list_began {
                    return Ok(expr);
                }
            } else {
                return Err(ParseError::UnexpectedToken(Token::CloseParen));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let mut parser = Parser::new("(add 1 2)".chars());
        let parsed_expr = parser.parse().unwrap();

        let expected_expr = Expr::List(Box::new(Cons {
            car: Expr::Sym(String::from("add")),
            cdr: Expr::List(Box::new(Cons {
                car: Expr::Num(1_f64),
                cdr: Expr::List(Box::new(Cons {
                    car: Expr::Num(2_f64),
                    cdr: Expr::Nil,
                })),
            })),
        }));

        assert_eq!(parsed_expr, expected_expr);
    }
}
