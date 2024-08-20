use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;

pub fn add(args: &Expr, env: &Env) -> EvalResult {
    let mut sum = 0_f64;
    let mut current_args = args;
    loop {
        match current_args {
            Expr::Nil => break,
            Expr::List(ref cons) => {
                if let Expr::Num(value) = eval(&cons.car, env)? {
                    sum += value;
                    current_args = &cons.cdr;
                } else {
                    return Err(String::from("Not a number!"));
                }
            }
            _ => return Err(String::from("Combination must be a proper list")),
        }
    }
    Ok(Expr::Num(sum))
}
