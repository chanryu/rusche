//! Rusche is a library for writing an interpreter for a Scheme-like language in Rust.
//! It lets you embed a Scheme-like interpreter into your Rust applications, allowing you
//! to use it as a scripting language or to create standalone interpreters. Rusche is
//! deliberately Scheme-*like*, not Scheme: it uses Scheme's syntax but keeps the core
//! language small.
//!
//! Tutorials for embedding Rusche in a host application:
//! - [Embedding the interpreter](https://github.com/chanryu/rusche/blob/main/docs/tutorials/embedding.md)
//! - [Writing a native function](https://github.com/chanryu/rusche/blob/main/docs/tutorials/native-functions.md)
//! - [Writing a foreign object wrapper](https://github.com/chanryu/rusche/blob/main/docs/tutorials/foreign.md)
//!
//! The [language reference](https://github.com/chanryu/rusche/blob/main/docs/language-reference.md)
//! documents the **core language** (built-ins and prelude). The example host
//! [rusche-cli](https://github.com/chanryu/rusche/tree/main/crates/rusche-cli) adds I/O
//! and vector and dict types — see
//! [rusche-cli.md](https://github.com/chanryu/rusche/blob/main/docs/rusche-cli.md).
//! Example `*.rsc` scripts target that host, not a bare evaluator.
//!
//! Also see the prelude in [src/prelude.rs](https://github.com/chanryu/rusche/blob/main/src/prelude.rs).

mod builtin;
mod prelude;

mod macros;

pub mod env;
pub mod error;
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
pub use error::Error;
pub use eval::{
    eval, eval_source, ErrorKind, EvalContext, EvalError, EvalResult, Evaluator, ForeignTracer,
    Frame, FrameKind, DEFAULT_GC_THRESHOLD, DEFAULT_MAX_CALL_DEPTH,
};
pub use expr::{intern, Expr, Foreign, NIL};
pub use lexer::{tokenize, LexError, Lexer};
pub use list::{cons, Cons, List, ListIter};
pub use parser::{ParseError, Parser};
pub use proc::{FormalArgs, NativeFunc, Proc};
pub use span::{Loc, Span};
pub use token::Token;
pub use utils::{arity_error, eval_into_foreign, eval_into_int, get_exact_1_arg, get_exact_2_args};
