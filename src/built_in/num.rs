use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::get_exact_one_arg;

fn binary_operation(
    proc_name: &str,
    args: &List,
    env: &Rc<Env>,
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
            _ => return Err(format!("{proc_name}: {arg} does not evaluate to a number!")),
        }
    }

    Ok(Expr::Num(result))
}

pub fn add(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn minus(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}

pub fn is_num(proc_name: &str, args: &List, _env: &Rc<Env>) -> EvalResult {
    if let Expr::Num(_) = get_exact_one_arg(proc_name, args)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;
    use crate::macros::list;

    #[test]
    fn test_add() {
        let env = Rc::new(Env::new());

        // (+ 1) => 1
        let args = list!(1);
        assert_eq!(add("", &args, &env), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(2, 1);
        assert_eq!(add("", &args, &env), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let env = Rc::new(Env::new());

        // (- 1) => -1
        let args = list!(1);
        assert_eq!(minus("", &args, &env), Ok(num(-1)));

        // (- 2 1) => 1
        let args = list!(2, 1);
        assert_eq!(minus("", &args, &env), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let env = Rc::new(Env::new());

        // (* 1) => 1
        let args = list!(1);
        assert_eq!(multiply("", &args, &env), Ok(num(1)));

        // (* 2 1) => 2
        let args = list!(2, 1);
        assert_eq!(multiply("", &args, &env), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = list!(3, 2, 1);
        assert_eq!(multiply("", &args, &env), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        let env = Rc::new(Env::new());

        // (/ 2) => 0.5
        let args = list!(2);
        assert_eq!(divide("", &args, &env), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(4, 2);
        assert_eq!(divide("", &args, &env), Ok(num(2)));
    }
}
