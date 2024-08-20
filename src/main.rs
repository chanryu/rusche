mod eval;
mod expr;
mod parser;
mod scanner;

use eval::eval;
use parser::Parser;

fn main() {
    let mut parser = Parser::new("(add 1 2)".chars());
    if let Ok(expr) = parser.parse() {
        let _ = eval(&expr);
    }
}
