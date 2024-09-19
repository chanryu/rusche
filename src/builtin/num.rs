use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::{eval_to_num, get_exact_1_arg, get_exact_2_args};

pub fn is_num(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    if let Expr::Num(_) = eval(get_exact_1_arg(proc_name, args)?, env)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

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
        let value = eval_to_num(proc_name, arg, env)?;
        if index == 0 && args.len() > 1 && !is_associative {
            result = value;
        } else {
            result = func(result, value);
        }
    }

    Ok(Expr::Num(result))
}

pub fn add(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn subtract(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    binary_operation(proc_name, args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}

pub fn modulo(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (lhs, rhs) = get_exact_2_args(proc_name, args)?;
    let lhs = eval_to_num(proc_name, lhs, env)?;
    let rhs = eval_to_num(proc_name, rhs, env)?;

    Ok(Expr::Num(lhs % rhs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::num;
    use crate::list::list;

    #[test]
    fn test_add() {
        let env = Env::for_unit_test();

        // (+ 1) => 1
        let args = list!(1);
        assert_eq!(add("", &args, &env), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(2, 1);
        assert_eq!(add("", &args, &env), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let env = Env::for_unit_test();

        // (- 1) => -1
        let args = list!(1);
        assert_eq!(subtract("", &args, &env), Ok(num(-1)));

        // (- 2 1) => 1
        let args = list!(2, 1);
        assert_eq!(subtract("", &args, &env), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let env = Env::for_unit_test();

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
        let env = Env::for_unit_test();

        // (/ 2) => 0.5
        let args = list!(2);
        assert_eq!(divide("", &args, &env), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(4, 2);
        assert_eq!(divide("", &args, &env), Ok(num(2)));
    }

    #[test]
    fn test_modulo() {
        let env = Env::for_unit_test();

        // (% 1 2) => 1
        assert_eq!(modulo("", &list!(1, 2), &env), Ok(Expr::from(1)));

        // (% 11 3) => 2
        assert_eq!(modulo("", &list!(11, 3), &env), Ok(num(2)));

        // (% 11 4) => 3
        assert_eq!(modulo("", &list!(11, 4), &env), Ok(num(3)));

        // (% 1) => error
        assert!(modulo("", &list!(1), &env).is_err());

        // (% 1 1 1) => error
        assert!(modulo("", &list!(1, 1, 1), &env).is_err());

        // (% "1" "2") => error
        assert!(modulo("", &list!("1", "2"), &env).is_err());
    }
}
