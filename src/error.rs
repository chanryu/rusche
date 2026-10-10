use std::error::Error as StdError;
use std::fmt;

use crate::eval::{EvalError, Frame};
use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::span::Span;

/// Unified error for the tokenize → parse → evaluate pipeline.
#[derive(Debug, PartialEq)]
pub enum Error {
    Lex(LexError),
    Parse(ParseError),
    Eval(EvalError),
}

impl Error {
    /// Returns the source span associated with this error, if any.
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Lex(e) => Some(e.span()),
            Error::Parse(e) => Some(e.span()),
            Error::Eval(e) => e.span,
        }
    }

    /// Bare message text without a leading span (for hosts that print their own location).
    pub fn message(&self) -> String {
        match self {
            Error::Lex(e) => e.message(),
            Error::Parse(e) => e.message(),
            Error::Eval(e) => e.message().to_string(),
        }
    }

    /// Optional secondary help text (evaluation errors only).
    pub fn help(&self) -> Option<&str> {
        match self {
            Error::Eval(e) => e.help(),
            _ => None,
        }
    }

    /// Call trace collected while unwinding (evaluation errors only).
    pub fn trace(&self) -> &[Frame] {
        match self {
            Error::Eval(e) => e.trace(),
            _ => &[],
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex(e) => write!(f, "{e}"),
            Error::Parse(e) => write!(f, "{e}"),
            Error::Eval(e) => write!(f, "{e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Lex(e) => Some(e),
            Error::Parse(e) => Some(e),
            Error::Eval(e) => Some(e),
        }
    }
}

impl From<LexError> for Error {
    fn from(value: LexError) -> Self {
        Error::Lex(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Parse(value)
    }
}

impl From<EvalError> for Error {
    fn from(value: EvalError) -> Self {
        Error::Eval(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{ErrorKind, EvalError, Frame, FrameKind};
    use crate::lexer::LexError;
    use crate::parser::ParseError;
    use crate::span::Loc;
    use crate::token::Token;
    use std::error::Error as StdError;

    fn span() -> Span {
        Span::new(Loc::new(0, 0), Loc::new(0, 3))
    }

    #[test]
    fn lex_parse_eval_message_span_display_and_source() {
        let lex: Error = LexError::IncompleteString(span()).into();
        assert_eq!(lex.span(), Some(span()));
        assert_eq!(lex.message(), "unterminated string literal");
        assert!(lex.help().is_none());
        assert!(lex.trace().is_empty());
        assert!(lex.to_string().contains("unterminated string literal"));
        assert!(lex.source().is_some());

        let num: Error = LexError::InvalidNumber("23abc".into(), span()).into();
        assert_eq!(num.message(), "invalid number literal `23abc`");
        assert!(num.to_string().contains("23abc"));

        let parse: Error = ParseError::UnexpectedToken(Token::CloseParen(Loc::new(0, 0))).into();
        assert_eq!(parse.message(), "unexpected `)` with no matching `(`");
        assert!(parse.help().is_none());
        assert!(parse.trace().is_empty());
        assert!(parse.source().is_some());
        assert!(parse.to_string().contains("unexpected `)`"));

        let eval_err = EvalError::new(ErrorKind::User, "boom")
            .with_span(Some(span()))
            .with_help("try again");
        let eval: Error = eval_err.into();
        assert_eq!(eval.span(), Some(span()));
        assert_eq!(eval.message(), "boom");
        assert_eq!(eval.help(), Some("try again"));
        assert!(eval.trace().is_empty());
        assert!(eval.to_string().starts_with("1:1"));
        assert!(eval.source().is_some());
    }

    #[test]
    fn eval_error_trace_and_display_without_span() {
        let mut err = EvalError::new(ErrorKind::Other, "plain");
        err.trace = vec![Frame {
            name: "f".into(),
            kind: FrameKind::Closure,
            call_site: None,
        }];
        let wrapped: Error = err.into();
        assert_eq!(wrapped.message(), "plain");
        assert_eq!(wrapped.to_string(), "plain");
        assert_eq!(wrapped.trace().len(), 1);
        assert_eq!(&*wrapped.trace()[0].name, "f");

        let from_string = EvalError::from("via From".to_string());
        assert_eq!(from_string.kind, ErrorKind::Other);
        assert_eq!(from_string.message(), "via From");
    }

    #[test]
    fn parse_error_messages_for_quote_forms() {
        use ParseError::IncompleteExpr;
        assert_eq!(
            IncompleteExpr(Token::Quote(Loc::default())).message(),
            "unexpected end of input after `'`"
        );
        assert_eq!(
            IncompleteExpr(Token::Quasiquote(Loc::default())).message(),
            "unexpected end of input after `` ` ``"
        );
        assert_eq!(
            IncompleteExpr(Token::Unquote(Loc::default())).message(),
            "unexpected end of input after `,`"
        );
        assert_eq!(
            IncompleteExpr(Token::UnquoteSplicing(Loc::default())).message(),
            "unexpected end of input after `,@`"
        );
        assert_eq!(
            IncompleteExpr(Token::OpenParen(Loc::default())).message(),
            "unexpected end of input: unclosed `(`"
        );
        // Fallback arm for other tokens stored as IncompleteExpr.
        assert_eq!(
            IncompleteExpr(Token::Sym("x".into(), span())).message(),
            "incomplete expression"
        );
        assert_eq!(
            ParseError::UnexpectedToken(Token::Sym("x".into(), span())).message(),
            "unexpected token: `x`"
        );
    }
}
