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
    let mut parser = Parser::new("(+ 1 (* 2 4))".chars());
    if let Ok(expr) = parser.parse() {
        let env = Env::new_root_env();
        let expr = env.get("+").unwrap();
        match eval(&expr, &env) {
            Ok(result) => {
                println!("{} => {}", expr, result);
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}
