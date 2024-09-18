use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalError};
use crate::expr::Expr;
use crate::list::List;

pub fn get_exact_one_arg<'a>(proc_name: &str, args: &'a List) -> Result<&'a Expr, EvalError> {
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

pub fn get_exact_two_args<'a>(
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

pub fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg) = item else {
            return Err(format!("{item} is not a symbol."));
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
}

pub fn eval_to_str(expr: &Expr, env: &Rc<Env>) -> Result<String, EvalError> {
    match eval(expr, env)? {
        Expr::Str(text) => Ok(text),
        _ => Err(format!("{expr} does not evaluate to a string.")),
    }
}

pub fn eval_to_num(expr: &Expr, env: &Rc<Env>) -> Result<f64, EvalError> {
    match eval(expr, env)? {
        Expr::Num(value) => Ok(value),
        _ => Err(format!("{expr} does not evaluate to a number.")),
    }
}
