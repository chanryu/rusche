use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;

fn binop(init_val: f64, func: fn(lhs: f64, rhs: f64) -> f64, args: &Expr, env: &Env) -> EvalResult {
    let mut result = init_val;
    let mut current_args = args;
    loop {
        match current_args {
            Expr::Nil => break,
            Expr::List(ref cons) => {
                if let Expr::Num(value) = eval(&cons.car, env)? {
                    result = func(result, value);
                    current_args = &cons.cdr;
                } else {
                    return Err(format!("{} is not a number!", cons.car));
                }
            }
            _ => return Err(String::from("Combination must be a proper list")),
        }
    }
    Ok(Expr::Num(result))
}

pub fn add(args: &Expr, env: &Env) -> EvalResult {
    binop(0_f64, |lhs, rhs| lhs + rhs, args, env)
}

pub fn minus(args: &Expr, env: &Env) -> EvalResult {
    binop(0_f64, |lhs, rhs| lhs - rhs, args, env)
}

pub fn mul(args: &Expr, env: &Env) -> EvalResult {
    binop(1_f64, |lhs, rhs| lhs * rhs, args, env)
}

pub fn div(args: &Expr, env: &Env) -> EvalResult {
    binop(1_f64, |lhs, rhs| lhs / rhs, args, env)
}
