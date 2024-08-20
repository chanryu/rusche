mod builtins;
mod env;
mod eval;
mod expr;
mod parser;
mod scanner;

use env::Env;
use eval::eval;
use parser::Parser;

fn main() {
    let mut parser = Parser::new("(add 1 2 4)".chars());
    if let Ok(expr) = parser.parse() {
        let env = Env::new_root_env();
        match eval(&expr, &env) {
            Ok(expr) => {
                println!("Result: {}", expr);
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}
