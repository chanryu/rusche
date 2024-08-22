use crate::expr::{Cons, Expr, NIL};
use crate::scanner::{ScanError, Scanner, Token};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    ScanError(ScanError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            ParseError::ScanError(error) => write!(f, "Error: \"{}\"", error),
        }
    }
}

type ParseResult = Result<Expr, ParseError>;

struct ParseContext {
    token: Option<Token>,
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
            let mut expr = match self.get_token()? {
                Token::OpenParen => {
                    self.begin_list(Token::OpenParen);
                    continue;
                }
                Token::Quote => {
                    self.begin_list(Token::Quote);
                    continue;
                }
                Token::CloseParen => self.end_list()?,
                Token::Sym(name) => Expr::Sym(name),
                Token::Str(text) => Expr::Str(text),
                Token::Num(value) => Expr::Num(value),
            };

            loop {
                if let Some(context) = self.contexts.last_mut() {
                    match context.token {
                        Some(Token::Quote) => {
                            self.contexts.pop();
                            expr =
                                Expr::new_list(Expr::new_sym("quote"), Expr::new_list(expr, NIL));
                            continue;
                        }
                        _ => {}
                    }
                    if context.car.is_none() {
                        context.car = Some(expr);
                    } else {
                        self.contexts.push(ParseContext {
                            token: None,
                            car: Some(expr),
                        });
                    }
                    break;
                } else {
                    return Ok(expr);
                }
            }
        }
    }

    fn get_token(&mut self) -> Result<Token, ParseError> {
        match self.scanner.get_token() {
            Ok(token) => Ok(token),
            Err(error) => Err(ParseError::ScanError(error)),
        }
    }

    fn begin_list(&mut self, token: Token) {
        self.contexts.push(ParseContext {
            token: Some(token),
            car: None,
        })
    }

    fn end_list(&mut self) -> ParseResult {
        let mut expr = NIL;
        while let Some(context) = self.contexts.pop() {
            if let Some(Token::Quote) = context.token {
                break;
            }
            if let Some(car) = context.car {
                expr = Expr::List(Some(Cons::new(car, expr)));
            } else {
                assert!(expr == NIL);
            }
            if context.token.is_some() {
                return Ok(expr);
            }
        }
        Err(ParseError::UnexpectedToken(Token::CloseParen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num<T>(value: T) -> Expr
    where
        T: Into<f64>,
    {
        Expr::new_num(value)
    }

    fn sym(name: &str) -> Expr {
        Expr::new_sym(name)
    }

    fn cons(car: Expr, cdr: Expr) -> Expr {
        Expr::new_list(car, cdr)
    }

    #[test]
    fn test_parser() {
        let mut parser = Parser::new("(add 1 2)".chars());
        let parsed_expr = parser.parse().unwrap();
        let expected_expr = cons(sym("add"), cons(num(1), cons(num(2), NIL)));
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_atom() {
        let mut parser = Parser::new("'1".chars());
        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = cons(sym("quote"), cons(num(1), NIL));
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_list() {
        let mut parser = Parser::new("'(1 2)".chars());
        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = cons(sym("quote"), cons(cons(num(1), cons(num(2), NIL)), NIL));
        assert_eq!(parsed_expr, expected_expr);
    }
}
