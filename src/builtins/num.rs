use crate::eval::{eval, Env, EvalResult};
use crate::expr::Expr;

fn binop(
    args: Vec<&Expr>,
    env: &Env,
    identity: f64,
    is_associative: bool,
    func: fn(lhs: f64, rhs: f64) -> f64,
) -> EvalResult {
    let mut result = identity;

    for (index, arg) in args.iter().enumerate() {
        match eval(&arg, env)? {
            Expr::Num(value) => {
                if index == 0 && args.len() > 1 && !is_associative {
                    result = value;
                } else {
                    result = func(result, value);
                }
            }
            _ => return Err(format!("{} is not a number!", arg)),
        }
    }

    Ok(Expr::Num(result))
}

pub fn add(args: &Expr, env: &Env) -> EvalResult {
    let args = args.splat();
    binop(args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn minus(args: &Expr, env: &Env) -> EvalResult {
    let args = args.splat();
    binop(args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(args: &Expr, env: &Env) -> EvalResult {
    let args = args.splat();
    binop(args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(args: &Expr, env: &Env) -> EvalResult {
    let args = args.splat();
    binop(args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}
