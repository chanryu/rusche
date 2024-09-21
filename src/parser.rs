use crate::expr::{intern, Expr};
use crate::list::{cons, list, List};
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

    pub fn with_tokens(tokens: Vec<Token>) -> Self {
        let mut parser = Self::new();
        parser.add_tokens(tokens);
        parser
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
                Token::OpenParen(_)
                | Token::Quote(_)
                | Token::Quasiquote(_)
                | Token::Unquote(_)
                | Token::UnquoteSplicing(_) => {
                    self.begin_list(token);
                    continue;
                }
                Token::CloseParen(_) => self.end_list(token)?,
                Token::Sym(_, name) => Expr::Sym(name),
                Token::Str(_, text) => Expr::Str(text),
                Token::Num(_, value) => Expr::Num(value),
            };

            loop {
                if let Some(context) = self.contexts.last_mut() {
                    if let Some(quote_name) = get_quote_name(context.token.as_ref()) {
                        self.contexts.pop();
                        expr = list!(intern(quote_name), expr).into();
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

    fn end_list(&mut self, token: Token) -> ParseResult {
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
        Err(ParseError::UnexpectedToken(token))
    }
}

fn get_quote_name(token: Option<&Token>) -> Option<&'static str> {
    match token {
        Some(Token::Quote(_)) => Some("quote"),
        Some(Token::Quasiquote(_)) => Some("quasiquote"),
        Some(Token::Unquote(_)) => Some("unquote"),
        Some(Token::UnquoteSplicing(_)) => Some("unquote-splicing"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Loc;

    // location agnostic token vector generator
    macro_rules! token_vec {
        ($($token_case:ident$(($value:expr))?),* $(,)?) => {{
            let mut v = Vec::new();
            $(
                v.push(Token::$token_case(Loc::new(1, 1)$(, $value)?));
            )*
            v
        }};
    }

    #[test]
    fn test_parser() {
        // (add 1 2)
        let mut parser = Parser::with_tokens(token_vec![
            OpenParen,
            Sym(String::from("add")),
            Num(1_f64),
            Num(2_f64),
            CloseParen,
        ]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(intern("add"), 1, 2).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_atom() {
        let loc = Loc::new(1, 0);

        // '1
        let mut parser = Parser::with_tokens(vec![Token::Quote(loc), Token::Num(loc, 1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(intern("quote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_list() {
        let mut parser = Parser::with_tokens(
            // '(1 2)
            token_vec![
                Quote,
                OpenParen,
                OpenParen,
                Num(1_f64),
                CloseParen,
                Num(2_f64),
                CloseParen,
            ],
        );

        let parsed_expr = parser.parse().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = list!(intern("quote"), list!(list!(1), 2)).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_other_quotes() {
        let mut parser = Parser::new();
        let loc = Loc::new(1, 0);

        // `1
        parser.add_tokens(token_vec![Quasiquote, Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(intern("quasiquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,1
        parser.add_tokens(token_vec![Unquote, Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(intern("unquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,@1
        parser.add_tokens(token_vec![UnquoteSplicing, Num(1_f64)]);

        let parsed_expr = parser.parse().unwrap();
        let expected_expr = list!(intern("unquote-splicing"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }
}
