//! Core language prelude: macros and helpers in Rusche, loaded via `include_str!`.

use crate::eval::{eval_source, EvalContext};

const PRELUDE: &[&str] = &[
    include_str!("macros.rsc"),
    include_str!("funcs.rsc"),
    include_str!("aliases.rsc"),
];

pub fn load_prelude(context: &EvalContext) {
    for src in PRELUDE {
        eval_src(src, context);
    }
}

fn eval_src(src: &str, context: &EvalContext) {
    // Prelude source locations are meaningless to users; strip them so errors
    // raised inside prelude code are attributed to the user's call site instead.
    if let Err(error) = eval_source(src, context, true) {
        use crate::error::Error;
        match error {
            Error::Lex(_) => panic!("Prelude tokenization failed:\n{src}"),
            Error::Parse(crate::parser::ParseError::IncompleteExpr(_)) => {
                panic!("Prelude parse failure - incomplete expression:\n{src}");
            }
            Error::Parse(crate::parser::ParseError::UnexpectedToken(token)) => {
                panic!("Prelude parse failure - unexpected token \"{token}\":\n{src}");
            }
            Error::Eval(_) => panic!("Prelude evaluation failed:\n{src}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;

    #[test]
    fn test_eval_src() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1)", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude tokenization failed")]
    fn test_eval_src_invalid_string() {
        let e = Evaluator::with_builtin();
        eval_src("\"x", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude evaluation failed")]
    fn test_eval_src_eval_failed() {
        let e = Evaluator::with_builtin();
        eval_src("(x)", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude parse failure - incomplete expression")]
    fn test_eval_src_incomplete_expr() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude parse failure - unexpected token")]
    fn test_eval_src_unexpected_token() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1))", e.context());
    }
}
