mod builtins;
mod env;
mod eval;
mod expr;
mod parser;
mod repl;
mod scanner;
mod token;

use repl::repl;

fn main() {
    repl();
}
