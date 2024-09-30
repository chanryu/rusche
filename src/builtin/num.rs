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
    use crate::list::list;

    #[test]
    fn test_add() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (+ 1) => 1
        let args = list!(1);
        assert_eq!(add("", &args, context), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(2, 1);
        assert_eq!(add("", &args, context), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (- 1) => -1
        let args = list!(1);
        assert_eq!(subtract("", &args, context), Ok(num(-1)));

        // (- 2 1) => 1
        let args = list!(2, 1);
        assert_eq!(subtract("", &args, context), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (* 1) => 1
        let args = list!(1);
        assert_eq!(multiply("", &args, context), Ok(num(1)));

        // (* 2 1) => 2
        let args = list!(2, 1);
        assert_eq!(multiply("", &args, context), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = list!(3, 2, 1);
        assert_eq!(multiply("", &args, context), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (/ 2) => 0.5
        let args = list!(2);
        assert_eq!(divide("", &args, context), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(4, 2);
        assert_eq!(divide("", &args, context), Ok(num(2)));
    }

    #[test]
    fn test_modulo() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (% 1 2) => 1
        assert_eq!(modulo("", &list!(1, 2), context), Ok(Expr::from(1)));

        // (% 11 3) => 2
        assert_eq!(modulo("", &list!(11, 3), context), Ok(num(2)));

        // (% 11 4) => 3
        assert_eq!(modulo("", &list!(11, 4), context), Ok(num(3)));

        // (% 1) => error
        assert!(modulo("", &list!(1), context).is_err());

        // (% 1 1 1) => error
        assert!(modulo("", &list!(1, 1, 1), context).is_err());

        // (% "1" "2") => error
        assert!(modulo("", &list!("1", "2"), context).is_err());
    }
}
