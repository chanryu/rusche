use crate::expr::Expr;
use crate::list::List;
use crate::token::Token;
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    NeedMoreToken,
    UnexpectedToken(Token),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NeedMoreToken => write!(f, "Ran out of tokens"),
            ParseError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
        }
    }
}

type ParseResult = Result<Expr, ParseError>;

struct ParseContext {
    token: Option<Token>,
    car: Option<Expr>,
}

pub struct Parser {
    tokens: VecDeque<Token>,
    contexts: Vec<ParseContext>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: VecDeque::new(),
            contexts: Vec::new(),
        }
    }

    pub fn is_parsing(&self) -> bool {
        !self.contexts.is_empty()
    }

    pub fn reset(&mut self) {
        self.tokens.clear();
        self.contexts.clear();
    }

    pub fn add_tokens<Iter>(&mut self, tokens: Iter)
    where
        Iter: IntoIterator<Item = Token>,
    {
        self.tokens.extend(tokens);
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
                            expr = Expr::List(List::new_cons(
                                Expr::Sym(String::from("quote")),
                                List::new_cons(expr, List::Nil),
                            ));
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
        if let Some(token) = self.tokens.pop_front() {
            Ok(token)
        } else {
            Err(ParseError::NeedMoreToken)
        }
    }

    fn begin_list(&mut self, token: Token) {
        self.contexts.push(ParseContext {
            token: Some(token),
            car: None,
        })
    }

    fn end_list(&mut self) -> ParseResult {
        let mut list = List::Nil;
        while let Some(context) = self.contexts.pop() {
            if let Some(Token::Quote) = context.token {
                break;
            }
            if let Some(car) = context.car {
                list = List::new_cons(car, list);
            }
            if context.token.is_some() {
                return Ok(list.to_expr());
            }
        }
        Err(ParseError::UnexpectedToken(Token::CloseParen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expr::{num, sym},
        list::cons,
    };

    #[test]
    fn test_parser() {
        let mut parser = Parser::new();

        // (add 1 2)
        parser.add_tokens([
            Token::OpenParen,
            Token::Sym(String::from("add")),
            Token::Num(1_f64),
            Token::Num(2_f64),
            Token::CloseParen,
        ]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = cons(sym("add"), cons(num(1), cons(num(2), List::Nil))).to_expr();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_atom() {
        let mut parser = Parser::new();

        // '1
        parser.add_tokens([Token::Quote, Token::Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = cons(sym("quote"), cons(num(1), List::Nil)).to_expr();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_list() {
        let mut parser = Parser::new();

        // '(1 2)
        parser.add_tokens([
            Token::Quote,
            Token::OpenParen,
            Token::Num(1_f64),
            Token::Num(2_f64),
            Token::CloseParen,
        ]);

        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = cons(
            sym("quote"),
            cons(cons(num(1), cons(num(2), List::Nil)).to_expr(), List::Nil),
        )
        .to_expr();
        assert_eq!(parsed_expr, expected_expr);
    }
}
