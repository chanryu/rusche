use std::any::Any;
use std::rc::Rc;

use crate::eval::{eval, EvalContext, EvalError};
use crate::expr::{intern, Expr};
use crate::list::{cons, List};

pub fn make_syntax_error(proc_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(intern(proc_name), args.clone())
    )
}

/// Get exactly one argument.
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
        return Err(format!("{proc_name} needs an argument."));
    };
    if iter.next().is_none() {
        Ok(arg)
    } else {
        Err(format!("{proc_name} expects only 1 argument."))
    }
}

/// Get exactly two arguments.
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
    let Some(arg1) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    let Some(arg2) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2))
    } else {
        Err(format!("{}: takes only two arguments", proc_name))
    }
}

/// Get exactly three arguments.
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
    let Some(arg1) = iter.next() else {
        return Err(format!("{}: requres 3 arguments", proc_name));
    };
    let Some(arg2) = iter.next() else {
        return Err(format!("{}: requres 3 arguments", proc_name));
    };
    let Some(arg3) = iter.next() else {
        return Err(format!("{}: requres 3 arguments", proc_name));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2, arg3))
    } else {
        Err(format!("{}: takes only 3 arguments", proc_name))
    }
}

pub fn get_2_or_3_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr, Option<&'a Expr>), EvalError> {
    let mut iter = args.iter();
    let Some(arg1) = iter.next() else {
        return Err(format!("{}: requres at least 2 arguments", proc_name));
    };
    let Some(arg2) = iter.next() else {
        return Err(format!("{}: requres at least 2 arguments", proc_name));
    };
    let Some(arg3) = iter.next() else {
        return Ok((arg1, arg2, None));
    };
    if iter.next().is_none() {
        Ok((arg1, arg2, Some(arg3)))
    } else {
        Err(format!("{}: takes up to 3 arguments", proc_name))
    }
}

pub fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg, _) = item else {
            return Err(format!("{item} is not a symbol."));
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
}

pub fn eval_into_str(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<String, EvalError> {
    match eval(expr, context)? {
        Expr::Str(text, _) => Ok(text),
        _ => Err(format!(
            "{proc_name}: {expr} does not evaluate to a string."
        )),
    }
}

pub fn eval_into_num(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<f64, EvalError> {
    match eval(expr, context)? {
        Expr::Num(value, _) => Ok(value),
        _ => Err(format!(
            "{proc_name}: {expr} does not evaluate to a number."
        )),
    }
}

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
        Err(format!(
            "{}: {} must be an integer, but got {}.",
            proc_name, arg_name, num
        ))
    }
}

pub fn eval_into_foreign(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<dyn Any>, EvalError> {
    match eval(expr, context)? {
        Expr::Foreign(object) => Ok(object),
        _ => Err(format!(
            "{proc_name}: {expr} does not evaluate to a foreign object."
        )),
    }
}
