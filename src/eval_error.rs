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
    NotCallable, // not a function or macro
    ArityMismatch,
    TypeMismatch,
    InvalidForm,

    Undefined, // TODO: remove this
}
