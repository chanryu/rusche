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
