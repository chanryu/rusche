use std::io::Write;

use rusp::{
    eval::{eval, EvalContext, EvalResult},
    expr::{Expr, NIL},
    list::List,
};

pub fn print(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

pub fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }
    Ok(Expr::from(input.trim()))
}
