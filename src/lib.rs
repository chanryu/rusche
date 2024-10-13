//! Rusche is a library for writing an interpreter for a Scheme-like language in Rust. It lets you embed a Scheme interpreter into your Rust applications, allowing you to use Scheme as a scripting language or to create standalone Scheme interpreters.
//!

mod builtin;
mod prelude;

pub mod env;
pub mod eval;
pub mod expr;
pub mod lexer;
pub mod list;
pub mod macros;
pub mod parser;
pub mod proc;
pub mod span;
pub mod token;
pub mod utils;
