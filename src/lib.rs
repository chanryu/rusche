//! Rusche is a library for writing an interpreter for a Scheme-like language in Rust.
//! It lets you embed a Scheme interpreter into your Rust applications, allowing you
//! to use Scheme as a scripting language or to create standalone Scheme interpreters.
//!
//! To learn how to implement or embed a Rusche interpreter, please have a look at
//! [rusche-cli](https://github.com/chanryu/rusche/tree/main/examples/rusche-cli).
//!
//! To learn more about the Rusche language, please have a look at *.rsc files in
//! the [examples](https://github.com/chanryu/rusche/tree/main/examples/) directory, or
//! have a look at the preludes in the [src/prelude.rs](https://github.com/chanryu/rusche/blob/main/src/prelude.rs) file.

mod builtin;
mod prelude;

mod macros;

pub mod env;
pub mod eval;
pub mod expr;
pub mod lexer;
pub mod list;
pub mod parser;
pub mod proc;
pub mod span;
pub mod token;
pub mod utils;

// Re-export public APIs
pub use env::Env;
pub use eval::{eval, eval_tail, EvalContext, EvalError, EvalResult, Evaluator};
pub use expr::{intern, Expr, Foreign, NIL};
pub use lexer::{tokenize, LexError, Lexer};
pub use list::{cons, Cons, List, ListIter};
pub use parser::{ParseError, Parser};
pub use proc::{NativeFunc, Proc};
pub use span::{Loc, Span};
pub use token::Token;
pub use utils::{eval_into_foreign, eval_into_int, get_exact_1_arg, get_exact_2_args};
