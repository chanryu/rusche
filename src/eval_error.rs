use std::fmt;

use crate::span::Span;

#[derive(Debug, PartialEq)]
pub struct EvalError {
    pub code: EvalErrorCode,
    pub message: String,
    pub span: Option<Span>,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}: {}", span, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EvalErrorCode {
    UndefinedSymbol,
    ArityMismatch,
    TypeMismatch, // TODO: split into more specific types -- CallableExpected, SymbolExpected, etc.
    InvalidForm,

    Undefined, // TODO: remove this and define something else for 3rd party errors
}
