use crate::expr::Expr;
use crate::list::{cons, List};
use crate::macros::list;
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
            let token = self.get_token()?;
            let mut expr = match token {
                Token::OpenParen
                | Token::Quote
                | Token::Quasiquote
                | Token::Unquote
                | Token::UnquoteSplicing => {
                    self.begin_list(token);
                    continue;
                }
                Token::CloseParen => self.end_list()?,
                Token::Sym(name) => Expr::Sym(name),
                Token::Str(text) => Expr::Str(text),
                Token::Num(value) => Expr::Num(value),
            };

            loop {
                if let Some(context) = self.contexts.last_mut() {
                    if let Some(quote_name) = get_quote_name(context.token.as_ref()) {
                        self.contexts.pop();
                        expr = list!(Expr::new_sym(quote_name), expr).into();
                        continue;
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
            if get_quote_name(context.token.as_ref()).is_some() {
                break;
            }
            if let Some(car) = context.car {
                list = cons(car, list);
            }
            if context.token.is_some() {
                return Ok(list.into());
            }
        }
        Err(ParseError::UnexpectedToken(Token::CloseParen))
    }
}

fn get_quote_name(token: Option<&Token>) -> Option<&'static str> {
    match token {
        Some(Token::Quote) => Some("quote"),
        Some(Token::Quasiquote) => Some("quasiquote"),
        Some(Token::Unquote) => Some("unquote"),
        Some(Token::UnquoteSplicing) => Some("unquote-splicing"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::sym;

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
        let expected_expr = list!(sym("add"), 1, 2).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_atom() {
        let mut parser = Parser::new();

        // '1
        parser.add_tokens([Token::Quote, Token::Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(sym("quote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_list() {
        let mut parser = Parser::new();

        // '(1 2)
        parser.add_tokens([
            Token::Quote,
            Token::OpenParen,
            Token::OpenParen,
            Token::Num(1_f64),
            Token::CloseParen,
            Token::Num(2_f64),
            Token::CloseParen,
        ]);

        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = list!(sym("quote"), list!(list!(1), 2)).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_other_quotes() {
        let mut parser = Parser::new();

        // `1
        parser.add_tokens([Token::Quasiquote, Token::Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(sym("quasiquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,1
        parser.add_tokens([Token::Unquote, Token::Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(sym("unquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,@1
        parser.add_tokens([Token::UnquoteSplicing, Token::Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(sym("unquote-splicing"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }
}
