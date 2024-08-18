mod expr;
mod parser;
mod scanner;

use parser::Parser;

fn main() {
    let mut parser = Parser::new("(add 1 2)".chars());
    let _ = parser.parse();
}
