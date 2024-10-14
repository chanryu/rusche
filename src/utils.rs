use std::any::Any;
use std::rc::Rc;

use crate::eval::{eval, EvalContext, EvalError};
use crate::expr::Expr;
use crate::list::List;

/// Get exactly one argument from a list.
///
/// Check if `args` contains extactly one argument. If so, return a reference
/// to the argument. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `args` - List of arguments.
///
/// # Example
///
/// ```
/// use rusche::{
///     expr::Expr,
///     utils::get_exact_1_arg,
///     list
/// };
///
/// let args = list!(1);
/// let result = get_exact_1_arg("add", &args);
/// assert_eq!(result, Ok(&Expr::from(1)));
///
/// let args = list!(1, 2);
/// let result = get_exact_1_arg("add", &args);
/// assert!(result.is_err());
/// ```
pub fn get_exact_1_arg<'a>(proc_name: &str, args: &'a List) -> Result<&'a Expr, EvalError> {
    let mut iter = args.iter();
    let Some(arg) = iter.next() else {
        return Err(EvalError::from(format!("{proc_name} needs an argument.")));
    };
    if iter.next().is_none() {
        Ok(arg)
    } else {
        Err(EvalError::from(format!(
            "{proc_name} expects only 1 argument."
        )))
    }
}

/// Get exactly two arguments from a list.
///
/// Check if `args` contains extactly two arguments. If so, return a tuple that contains
/// references to the two arguments. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `args` - List of arguments.
///
/// # Example
///
/// ```
/// use rusche::{
///     expr::Expr,
///     utils::get_exact_2_args,
///     list
/// };
///
/// let args = list!(1, 2);
/// let result = get_exact_2_args("add", &args);
/// assert_eq!(result, Ok((&Expr::from(1), &Expr::from(2))));
///
/// let args = list!(1);
/// let result = get_exact_2_args("add", &args);
/// assert!(result.is_err());
/// ```
pub fn get_exact_2_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();

    let arg1 = iter.next();
    let arg2 = iter.next();
    let arg3 = iter.next();

    match (arg1, arg2, arg3) {
        (Some(arg1), Some(arg2), None) => Ok((arg1, arg2)),
        (Some(_), Some(_), Some(_)) => Err(EvalError::from(format!(
            "{proc_name}: takes only two arguments"
        ))),
        _ => Err(EvalError::from(format!(
            "{proc_name}: requres two arguments"
        ))),
    }
}

/// Get exactly three arguments from a list.
///
/// Check if `args` contains extactly three arguments. If so, return a tuple that contains
/// references to the three arguments. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `args` - List of arguments.
///
/// # Example
///
/// ```
/// use rusche::{
///     expr::Expr,
///     utils::get_exact_3_args,
///     list
/// };
///
/// let args = list!(1, 2, 3);
/// let result = get_exact_3_args("add", &args);
/// assert_eq!(result, Ok((&Expr::from(1), &Expr::from(2), &Expr::from(3))));
///
/// let args = list!(1, 2, 3, 4);
/// let result = get_exact_3_args("add", &args);
/// assert!(result.is_err());
/// ```
pub fn get_exact_3_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();

    let arg1 = iter.next();
    let arg2 = iter.next();
    let arg3 = iter.next();
    let arg4 = iter.next();

    match (arg1, arg2, arg3, arg4) {
        (Some(arg1), Some(arg2), Some(arg3), None) => Ok((arg1, arg2, arg3)),
        (Some(_), Some(_), Some(_), Some(_)) => Err(EvalError::from(format!(
            "{proc_name}: takes only two arguments"
        ))),
        _ => Err(EvalError::from(format!(
            "{proc_name}: requres two arguments"
        ))),
    }
}

/// Get two or three arguments from a list.
///
/// Check if `args` contains two or three arguments. If so, return a tuple that contains
/// references to the two arguments and optional 3rd argument. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `args` - List of arguments.
///
/// # Example
///
/// ```
/// use rusche::{
///     expr::Expr,
///     utils::get_2_or_3_args,
///     list
/// };
///
/// let args = list!(1, 2);
/// let result = get_2_or_3_args("add", &args);
/// assert_eq!(result, Ok((&Expr::from(1), &Expr::from(2), None)));
///
/// let args = list!(1, 2, 3);
/// let result = get_2_or_3_args("add", &args);
/// assert_eq!(result, Ok((&Expr::from(1), &Expr::from(2), Some(&Expr::from(3)))));
/// ```
pub fn get_2_or_3_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr, Option<&'a Expr>), EvalError> {
    let mut iter = args.iter();

    let arg1 = iter.next();
    let arg2 = iter.next();
    let arg3 = iter.next();
    let arg4 = iter.next();

    match (arg1, arg2, arg3, arg4) {
        (Some(arg1), Some(arg2), arg3, None) => Ok((arg1, arg2, arg3)),
        (Some(_), Some(_), Some(_), Some(_)) => Err(EvalError::from(format!(
            "{proc_name}: takes only up to 3 arguments"
        ))),
        _ => Err(EvalError::from(format!(
            "{proc_name}: requres at least 2 arguments"
        ))),
    }
}

/// Make a vector of symbol names from a list of arguments.
///
/// Check if `list` contains only symbols. If so, return a vector of the symbols.
/// Otherwise, return an error message. This function can be used to extract formal
/// arguments when implementing a function-like special form such as `lambda` or `defmacro`.
pub fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg, _) = item else {
            return Err(EvalError {
                message: format!("{item} is not a symbol."),
                span: item.span(),
            });
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
}

/// Evaluate an expression into a string.
///
/// Check if `expr` evaluates to a string. If so, return the string. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `expr` - Expression to evaluate.
/// * `context` - Evaluation context.
///
/// # Example
///
/// ```
/// use rusche::{
///     eval::Evaluator,
///     expr::Expr,
///     utils::eval_into_str,
/// };
///
/// let evaluator = Evaluator::new();
/// let expr = Expr::from("hello");
/// let result = eval_into_str("test", &expr, evaluator.context());
/// assert_eq!(result, Ok("hello".to_string()));
/// ```
pub fn eval_into_str(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<String, EvalError> {
    match eval(expr, context)? {
        Expr::Str(text, _) => Ok(text),
        _ => Err(EvalError {
            message: format!("{proc_name}: `{expr}` does not evaluate to a string."),
            span: expr.span(),
        }),
    }
}

/// Evaluate an expression into a number (`f64``).
///
/// Check if `expr` evaluates to a number. If so, return the number. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `expr` - Expression to evaluate.
/// * `context` - Evaluation context.
///
/// # Example
///
/// ```
/// use rusche::{
///     eval::Evaluator,
///     expr::Expr,
///     utils::eval_into_num,
/// };
///
/// let evaluator = Evaluator::new();
/// let expr = Expr::from(12e-3);
/// let result = eval_into_num("test", &expr, evaluator.context());
/// assert_eq!(result, Ok(12e-3));
/// ```
pub fn eval_into_num(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<f64, EvalError> {
    match eval(expr, context)? {
        Expr::Num(value, _) => Ok(value),
        _ => Err(EvalError {
            message: format!("{proc_name}: `{expr}` does not evaluate to a number."),
            span: expr.span(),
        }),
    }
}

/// Evaluate an expression into an integer (`i32`).
///
/// Check if `expr` evaluates to `f64`` with `fract() == 0``. If so, return the number
/// as i32. Otherwise, return an error message.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `arg_name` - Name of the argument that we want to evaluate to an integer.
/// * `expr` - Expression to evaluate.
/// * `context` - Evaluation context.
///
/// # Example
///
/// ```
/// use rusche::{
///     eval::Evaluator,
///     expr::Expr,
///     utils::eval_into_int,
/// };
///
/// let evaluator = Evaluator::new();
///
/// let expr = Expr::from(12);
/// let result = eval_into_int("test", "index", &expr, evaluator.context());
/// assert_eq!(result, Ok(12));
///
/// let expr = Expr::from(12.0);
/// let result = eval_into_int("test", "index", &expr, evaluator.context());
/// assert_eq!(result, Ok(12));
///
/// let expr = Expr::from(12.5);
/// let result = eval_into_int("test", "index", &expr, evaluator.context());
/// assert!(result.is_err());
/// ```
pub fn eval_into_int(
    proc_name: &str,
    arg_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<i32, EvalError> {
    let num = eval_into_num(proc_name, expr, context)?;

    if num.fract() == 0.0 {
        Ok(num as i32)
    } else {
        Err(EvalError {
            message: format!(
                "{}: {} must be an integer, but got {}.",
                proc_name, arg_name, num
            ),
            span: expr.span(),
        })
    }
}

/// Evaluate an expression into a foreign object.
///
/// Check if `expr` evaluates to a foreign object (`Expr::Foreign`). If so, return
/// the object (`Rc<dyn Any>`). Otherwise, return an error message.
/// The caller of this function can downcase the object to the expected type.
///
/// # Arguments
///
/// * `proc_name` - Name of the procedure who is calling this function.
/// * `expr` - Expression to evaluate.
/// * `context` - Evaluation context.
///
/// # Example
///
/// ```
/// use std::{any::Any, rc::Rc};
/// use rusche::{
///     eval::Evaluator,
///     expr::Expr,
///     utils::eval_into_foreign,
/// };
///
/// let evaluator = Evaluator::new();
/// let context = evaluator.context();
/// let expr = Expr::Foreign(Rc::new(Vec::<i32>::new()));
/// let object = eval_into_foreign("test", &expr, context).unwrap();
/// assert!(object.downcast::<Vec<i32>>().is_ok());
/// ```
pub fn eval_into_foreign(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<dyn Any>, EvalError> {
    match eval(expr, context)? {
        Expr::Foreign(object) => Ok(object),
        _ => Err(EvalError {
            message: format!("{proc_name}: `{expr}` does not evaluate to a foreign object."),
            span: expr.span(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::list;

    #[test]
    fn test_get_exact_1_arg() {
        let args = list!(1);
        let result = get_exact_1_arg("add", &args);
        assert_eq!(result, Ok(&num(1)));

        let args = list!();
        let result = get_exact_1_arg("add", &args);
        assert!(result.is_err());

        let args = list!(1, 2);
        let result = get_exact_1_arg("add", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exact_2_args() {
        let args = list!(1, 2);
        let result = get_exact_2_args("add", &args);
        assert_eq!(result, Ok((&num(1), &num(2))));

        let args = list!(1);
        let result = get_exact_2_args("add", &args);
        assert!(result.is_err());

        let args = list!(1, 2, 3);
        let result = get_exact_2_args("add", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exact_3_args() {
        let args = list!(1, 2, 3);
        let result = get_exact_3_args("add", &args);
        assert_eq!(result, Ok((&num(1), &num(2), &num(3))));

        let args = list!(1, 2);
        let result = get_exact_3_args("add", &args);
        assert!(result.is_err());

        let args = list!(1, 2, 3, 4);
        let result = get_exact_3_args("add", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_into_str() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let result = eval_into_str("test", &Expr::from("hello"), context);
        assert_eq!(result, Ok("hello".to_string()));

        let result = eval_into_str("test", &Expr::from(1), context);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_into_num() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let result = eval_into_num("test", &Expr::from(1), context);
        assert_eq!(result, Ok(1_f64));

        let result = eval_into_num("test", &Expr::from("1"), context);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_into_int() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let result = eval_into_int("test", "index", &Expr::from(1), context);
        assert_eq!(result, Ok(1));

        let result = eval_into_int("test", "index", &Expr::from(1.1), context);
        assert!(result.is_err());

        let result = eval_into_int("test", "index", &Expr::from("1"), context);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_into_foreign() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let expr = Expr::Foreign(Rc::new(Vec::<i32>::new()));
        let object = eval_into_foreign("test", &expr, context).unwrap();
        assert!(object.downcast::<Vec<i32>>().is_ok());

        assert!(eval_into_foreign("test", &Expr::from(1), context).is_err());
        assert!(eval_into_foreign("test", &Expr::from("str"), context).is_err());
        assert!(eval_into_foreign("test", &intern("sym"), context).is_err());
    }
}
