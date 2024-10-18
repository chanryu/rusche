use crate::{
    eval::{eval, EvalContext, EvalResult},
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

    for (index, arg) in args.iter().enumerate() {
        let value = eval_into_num(proc_name, arg, context)?;
        if index == 0 && args.len() > 1 && !is_associative {
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

pub fn greater(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    logical_operation(proc_name, args, context, |lhs, rhs| lhs > rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::test_utils::num;
    use crate::expr::{intern, NIL};
    use crate::macros::*;

    #[test]
    fn test_is_num() {
        setup_native_proc_test!(is_num);

        // (is-num 1) => #t
        let args = list!(1);
        assert_eq!(is_num(args), Ok(num(1)));

        // (is-num "str") => #f
        let args = list!("str");
        assert_eq!(is_num(args), Ok(NIL));

        // (is-num 'sym) => #f
        let args = list!(list!(intern("quote"), intern("sym")));
        assert_eq!(is_num(args), Ok(NIL));

        // (is-num '()) => #f
        let args = list!(list!(intern("quote"), list!()));
        assert_eq!(is_num(args), Ok(NIL));

        // (is-num '(1 2 3)) => #f
        let args = list!(list!(intern("quote"), list!(1, 2, 3)));
        assert_eq!(is_num(args), Ok(NIL));
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

        // (< 1 2) => #t
        assert_eq!(less(list!(1, 2)), Ok(true.into()));

        // (< 1 1) => #f
        assert_eq!(less(list!(1, 1)), Ok(false.into()));

        // (< 2 1) => #f
        assert_eq!(less(list!(2, 1)), Ok(false.into()));
    }

    #[test]
    fn test_greater() {
        setup_native_proc_test!(greater);

        // (> 1 2) => #t
        assert_eq!(greater(list!(1, 2)), Ok(false.into()));

        // (> 1 1) => #f
        assert_eq!(greater(list!(1, 1)), Ok(false.into()));

        // (> 2 1) => #f
        assert_eq!(greater(list!(2, 1)), Ok(true.into()));
    }
}
