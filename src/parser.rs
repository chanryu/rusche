use crate::expr::{intern, Expr};
use crate::list::{cons, List};
use crate::macros::list;
use crate::span::Span;
use crate::token::Token;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    IncompleteExpr(Token),
    UnexpectedToken(Token),
}

type ParseResult = Result<Option<Expr>, ParseError>;

struct ParseContext {
    token: Option<Token>,
    car: Option<Expr>,
}

/// A parser that converts a sequence of tokens into expressions.
pub struct Parser {
    tokens: VecDeque<Token>,
    contexts: Vec<ParseContext>,
}

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self {
            tokens: VecDeque::new(),
            contexts: Vec::new(),
        }
    }

    /// Create a new parser with the given tokens.
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
            let Some(token) = self.get_token() else {
                return if self.contexts.is_empty() {
                    Ok(None)
                } else {
                    Err(ParseError::IncompleteExpr(self.get_expr_begin_token()))
                };
            };

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
                Token::Sym(name, span) => Expr::Sym(name, Some(span)),
                Token::Str(text, span) => Expr::Str(text, Some(span)),
                Token::Num(value, span) => Expr::Num(value, Some(span)),
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
                    return Ok(Some(expr));
                }
            }
        }
    }

    fn get_token(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn begin_list(&mut self, token: Token) {
        self.contexts.push(ParseContext {
            token: Some(token),
            car: None,
        })
    }

    fn end_list(&mut self, token: Token) -> Result<Expr, ParseError> {
        let mut list = List::Nil;
        while let Some(context) = self.contexts.pop() {
            if get_quote_name(context.token.as_ref()).is_some() {
                break;
            }
            if let Some(car) = context.car {
                list = cons(car, list);
            }
            if let Some(begin_token) = context.token {
                let expr_span = Span {
                    begin: begin_token.span().begin,
                    end: token.span().end,
                };
                return Ok(Expr::List(list, Some(expr_span)));
            }
        }
        Err(ParseError::UnexpectedToken(token)) // dangling ')'
    }

    fn get_expr_begin_token(&self) -> Token {
        assert!(!self.contexts.is_empty());

        // Find the first token that started the current expression
        for context in self.contexts.iter().rev() {
            if let Some(token) = context.token.as_ref() {
                return token.clone();
            }
        }
        panic!("No token found for the current expression");
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

fn get_quote_name(token: Option<&Token>) -> Option<&'static str> {
    use crate::builtin::quote::{QUASIQUOTE, QUOTE, UNQUOTE, UNQUOTE_SPLICING};
    match token {
        Some(Token::Quote(_)) => Some(QUOTE),
        Some(Token::Quasiquote(_)) => Some(QUASIQUOTE),
        Some(Token::Unquote(_)) => Some(UNQUOTE),
        Some(Token::UnquoteSplicing(_)) => Some(UNQUOTE_SPLICING),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::tok;

    #[test]
    fn test_parser() {
        // (add 1 2)
        let mut parser = Parser::with_tokens(vec![
            tok!(OpenParen),
            tok!(Sym("add")),
            tok!(Num(1)),
            tok!(Num(2)),
            tok!(CloseParen),
        ]);

        let parsed_expr = parser.parse().unwrap().unwrap();
        let expected_expr = list!(intern("add"), 1, 2).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_reset() {
        let mut parser = Parser::default();

        // add "(1" -- incomplete expression
        parser.add_tokens(vec![tok!(OpenParen), tok!(Num(1_f64))]);

        // error on incomplete expression
        assert_eq!(
            parser.parse(),
            Err(ParseError::IncompleteExpr(tok!(OpenParen)))
        );
        assert!(parser.is_parsing());

        // cannot recover from previous error
        assert_eq!(
            parser.parse(),
            Err(ParseError::IncompleteExpr(tok!(OpenParen)))
        );
        assert!(parser.is_parsing());

        // reset tokens and contexts
        parser.reset();

        assert!(!parser.is_parsing());

        // verify that parser is reset
        assert_eq!(parser.parse(), Ok(None));
    }

    #[test]
    fn test_parser_quote_atom() {
        // '1
        let mut parser = Parser::with_tokens(vec![tok!(Quote), tok!(Num(1))]);

        let parsed_expr = parser.parse().unwrap().unwrap();
        let expected_expr = list!(intern("quote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_quote_list() {
        let mut parser = Parser::with_tokens(
            // '(1 2)
            vec![
                tok!(Quote),
                tok!(OpenParen),
                tok!(OpenParen),
                tok!(Num(1_f64)),
                tok!(CloseParen),
                tok!(Num(2_f64)),
                tok!(CloseParen),
            ],
        );

        let parsed_expr = parser.parse().unwrap().unwrap();
        print!("{}", parsed_expr);
        let expected_expr = list!(intern("quote"), list!(list!(1), 2)).into();
        assert_eq!(parsed_expr, expected_expr);
    }

    #[test]
    fn test_parser_other_quotes() {
        let mut parser = Parser::default();

        // `1
        parser.add_tokens(vec![tok!(Quasiquote), tok!(Num(1_f64))]);

        let parsed_expr = parser.parse().unwrap().unwrap();
        let expected_expr = list!(intern("quasiquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,1
        parser.add_tokens(vec![tok!(Unquote), tok!(Num(1_f64))]);

        let parsed_expr = parser.parse().unwrap().unwrap();
        let expected_expr = list!(intern("unquote"), 1).into();
        assert_eq!(parsed_expr, expected_expr);

        // ,@1
        parser.add_tokens(vec![tok!(UnquoteSplicing), tok!(Num(1_f64))]);

        let parsed_expr = parser.parse().unwrap().unwrap();
        let expected_expr = list!(intern("unquote-splicing"), 1).into();
        assert_eq!(parsed_expr, expected_expr);
    }
}
