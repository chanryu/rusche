use crate::{
    eval::{eval, ErrorKind, EvalContext, EvalError, EvalResult},
    expr::Expr,
    list::List,
    utils::{eval_into_num, get_exact_1_arg, get_exact_2_args},
};

pub fn is_num(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Num(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

fn binary_operation(
    proc_name: &str,
    args: &List,
    context: &EvalContext,
    identity: f64,
    is_associative: bool,
    func: fn(lhs: f64, rhs: f64) -> f64,
) -> EvalResult {
    let mut result = identity;
    let len = args.len();

    for (index, arg) in args.iter().enumerate() {
        let value = eval_into_num(proc_name, arg, context)?;
        if index == 0 && len > 1 && !is_associative {
            result = value;
        } else {
            result = func(result, value);
        }
    }

    Ok(Expr::Num(result, None))
}

pub fn add(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    binary_operation(proc_name, args, context, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn subtract(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    binary_operation(proc_name, args, context, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    binary_operation(proc_name, args, context, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    binary_operation(proc_name, args, context, 1_f64, false, |lhs, rhs| lhs / rhs)
}

pub fn modulo(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (lhs, rhs) = get_exact_2_args(proc_name, args)?;
    let lhs = eval_into_num(proc_name, lhs, context)?;
    let rhs = eval_into_num(proc_name, rhs, context)?;

    Ok(Expr::Num(lhs % rhs, None))
}

fn logical_operation(
    proc_name: &str,
    args: &List,
    context: &EvalContext,
    func: fn(lhs: f64, rhs: f64) -> bool,
) -> EvalResult {
    let (lhs, rhs) = get_exact_2_args(proc_name, args)?;
    Ok(Expr::from(func(
        eval_into_num(proc_name, lhs, context)?,
        eval_into_num(proc_name, rhs, context)?,
    )))
}

pub fn less(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    logical_operation(proc_name, args, context, |lhs, rhs| lhs < rhs)
}

fn unary_num(
    proc_name: &str,
    args: &List,
    context: &EvalContext,
    func: fn(f64) -> f64,
) -> EvalResult {
    let value = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::Num(func(value), None))
}

pub fn sqrt(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    unary_num(proc_name, args, context, f64::sqrt)
}

pub fn exp(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    unary_num(proc_name, args, context, f64::exp)
}

/// Natural logarithm of one argument, or log base `b` of `x` with two arguments.
pub fn log(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    match args.len() {
        1 => unary_num(proc_name, args, context, f64::ln),
        2 => {
            let (x, base) = get_exact_2_args(proc_name, args)?;
            let x = eval_into_num(proc_name, x, context)?;
            let base = eval_into_num(proc_name, base, context)?;
            Ok(Expr::Num(x.log(base), None))
        }
        n => Err(EvalError::new(
            ErrorKind::Arity,
            format!("{proc_name}: expected 1 or 2 arguments, got {n}"),
        )),
    }
}

pub fn expt(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (base, exp) = get_exact_2_args(proc_name, args)?;
    let base = eval_into_num(proc_name, base, context)?;
    let exp = eval_into_num(proc_name, exp, context)?;
    Ok(Expr::Num(base.powf(exp), None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::*;

    #[test]
    fn test_is_num() {
        setup_native_proc_test!(is_num);

        // (is-num 1) => true
        let args = list!(1);
        assert_eq!(is_num(args), Ok(true.into()));

        // (is-num "str") => false
        let args = list!("str");
        assert_eq!(is_num(args), Ok(false.into()));

        // (is-num 'sym) => false
        let args = list!(list!(intern("quote"), intern("sym")));
        assert_eq!(is_num(args), Ok(false.into()));

        // (is-num '()) => false
        let args = list!(list!(intern("quote"), list!()));
        assert_eq!(is_num(args), Ok(false.into()));

        // (is-num '(1 2 3)) => false
        let args = list!(list!(intern("quote"), list!(1, 2, 3)));
        assert_eq!(is_num(args), Ok(false.into()));
    }

    #[test]
    fn test_add() {
        setup_native_proc_test!(add);

        // (+ 1) => 1
        let args = list!(1);
        assert_eq!(add(args), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(2, 1);
        assert_eq!(add(args), Ok(num(3)));

        // (+ 3 2 1) => 6
        let args = list!(3, 2, 1);
        assert_eq!(add(args), Ok(num(6)));
    }

    #[test]
    fn test_minus() {
        setup_native_proc_test!(subtract);

        // (- 1) => -1
        let args = list!(1);
        assert_eq!(subtract(args), Ok(num(-1)));

        // (- -1) => 1
        let args = list!(-1);
        assert_eq!(subtract(args), Ok(num(1)));

        // (- 2 1) => 1
        let args = list!(2, 1);
        assert_eq!(subtract(args), Ok(num(1)));

        // (- 1 2) => -1
        let args = list!(1, 2);
        assert_eq!(subtract(args), Ok(num(-1)));
    }

    #[test]
    fn test_multiply() {
        setup_native_proc_test!(multiply);

        // (* 1) => 1
        let args = list!(1);
        assert_eq!(multiply(args), Ok(num(1)));

        // (* 2 1) => 2
        let args = list!(2, 1);
        assert_eq!(multiply(args), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = list!(3, 2, 1);
        assert_eq!(multiply(args), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        setup_native_proc_test!(divide);

        // (/ 2) => 0.5
        let args = list!(2);
        assert_eq!(divide(args), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(4, 2);
        assert_eq!(divide(args), Ok(num(2)));
    }

    #[test]
    fn test_modulo() {
        setup_native_proc_test!(modulo);

        // (% 1 2) => 1
        assert_eq!(modulo(list!(1, 2)), Ok(Expr::from(1)));

        // (% 11 3) => 2
        assert_eq!(modulo(list!(11, 3)), Ok(num(2)));

        // (% 11 4) => 3
        assert_eq!(modulo(list!(11, 4)), Ok(num(3)));

        // (% 1) => error
        assert!(modulo(list!(1)).is_err());

        // (% 1 1 1) => error
        assert!(modulo(list!(1, 1, 1)).is_err());

        // (% "1" "2") => error
        assert!(modulo(list!("1", "2")).is_err());
    }

    #[test]
    fn test_less() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();
        let less = |args| less("", &args, context);

        // (< 1 2) => true
        assert_eq!(less(list!(1, 2)), Ok(true.into()));

        // (< 1 1) => false
        assert_eq!(less(list!(1, 1)), Ok(false.into()));

        // (< 2 1) => false
        assert_eq!(less(list!(2, 1)), Ok(false.into()));
    }

    #[test]
    fn test_sqrt_exp_log_expt() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();
        let sqrt = |args| sqrt("sqrt", &args, context);
        let exp = |args| exp("exp", &args, context);
        let log = |args| log("log", &args, context);
        let expt = |args| expt("expt", &args, context);

        assert_eq!(sqrt(list!(9)), Ok(num(3)));
        assert_eq!(expt(list!(2, 10)), Ok(num(1024)));
        assert_eq!(log(list!(1)), Ok(num(0)));
        assert_eq!(log(list!(8, 2)), Ok(num(3)));

        let e = exp(list!(1)).unwrap();
        let Expr::Num(value, _) = e else {
            panic!("expected number");
        };
        assert!((value - std::f64::consts::E).abs() < 1e-10);

        assert!(sqrt(list!()).is_err());
        assert!(sqrt(list!(1, 2)).is_err());
        assert!(expt(list!(2)).is_err());
        assert!(log(list!(1, 2, 3)).is_err());
        assert!(sqrt(list!("9")).is_err());
    }
}
