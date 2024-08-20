use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;

pub fn add(args: &Expr, env: &Env) -> EvalResult {
    let mut sum = 0_f64;
    let mut args = args;
    loop {
        match args {
            Expr::Nil => break,
            Expr::List(ref cons) => {
                if let Expr::Num(value) = eval(&cons.car, env)? {
                    sum += value;
                    args = &cons.cdr;
                } else {
                    return Err(String::from("Not a number!"));
                }
            }
            _ => return Err(String::from("Combination must be a proper list")),
        }
    }
    Ok(Expr::Num(sum))
}
