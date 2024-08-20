mod builtins;
mod eval;
mod expr;
mod parser;
mod scanner;

use builtins::num::add;
use eval::{eval, Env};
use expr::Expr;
use parser::Parser;

fn main() {
    let mut parser = Parser::new("(add 1 2)".chars());
    if let Ok(expr) = parser.parse() {
        let mut env = Env::new();
        env.set("add", Expr::Proc(add));
        match eval(&expr, &env) {
            Ok(_) => {
                println!("Eval success!");
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}
