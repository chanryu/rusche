use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::get_exact_one_arg;

fn binary_operation(
    func_name: &str,
    args: &List,
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
            _ => return Err(format!("{func_name}: {arg} does not evaluate to a number!")),
        }
    }

    Ok(Expr::Num(result))
}

pub fn add(func_name: &str, args: &List, env: &Env) -> EvalResult {
    binary_operation(func_name, args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn minus(func_name: &str, args: &List, env: &Env) -> EvalResult {
    binary_operation(func_name, args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(func_name: &str, args: &List, env: &Env) -> EvalResult {
    binary_operation(func_name, args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(func_name: &str, args: &List, env: &Env) -> EvalResult {
    binary_operation(func_name, args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}

pub fn is_num(func_name: &str, args: &List, _env: &Env) -> EvalResult {
    if let Expr::Num(_) = get_exact_one_arg(func_name, args)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;
    use crate::list::{cons, list};

    #[test]
    fn test_add() {
        let env = Env::new();

        // (+ 1) => 1
        let args = list!(num(1));
        assert_eq!(add("", &args, &env), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(num(2), num(1));
        assert_eq!(add("", &args, &env), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let env = Env::new();

        // (- 1) => -1
        let args = list!(num(1));
        assert_eq!(minus("", &args, &env), Ok(num(-1)));

        // (- 2 1) => 1
        let args = list!(num(2), num(1));
        assert_eq!(minus("", &args, &env), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let env = Env::new();

        // (* 1) => 1
        let args = list!(num(1));
        assert_eq!(multiply("", &args, &env), Ok(num(1)));

        // (* 2 1) => 2
        let args = list!(num(2), num(1));
        assert_eq!(multiply("", &args, &env), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = list!(num(3), num(2), num(1));
        assert_eq!(multiply("", &args, &env), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        let env = Env::new();

        // (/ 2) => 0.5
        let args = list!(num(2));
        assert_eq!(divide("", &args, &env), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(num(4), num(2));
        assert_eq!(divide("", &args, &env), Ok(num(2)));
    }
}
