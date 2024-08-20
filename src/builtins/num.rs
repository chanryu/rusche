use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;

type BinaryFunc = fn(lhs: f64, rhs: f64) -> f64;

fn binary_operation(init_val: f64, func: BinaryFunc, args: &Expr, env: &Env) -> EvalResult {
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
    binary_operation(0_f64, |lhs, rhs| lhs + rhs, args, env)
}

pub fn minus(args: &Expr, env: &Env) -> EvalResult {
    binary_operation(0_f64, |lhs, rhs| lhs - rhs, args, env)
}

pub fn multiply(args: &Expr, env: &Env) -> EvalResult {
    binary_operation(1_f64, |lhs, rhs| lhs * rhs, args, env)
}

pub fn divide(args: &Expr, env: &Env) -> EvalResult {
    binary_operation(1_f64, |lhs, rhs| lhs / rhs, args, env)
}
