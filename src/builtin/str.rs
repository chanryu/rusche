use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::{
    eval_to_num, eval_to_str, get_exact_3_args, get_exact_one_arg, get_exact_two_args,
};

pub fn is_str(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    if let Expr::Str(_) = eval(get_exact_one_arg(proc_name, args)?, env)? {
        Ok(Expr::from(true))
    } else {
        Ok(Expr::from(false))
    }
}

pub fn compare(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (arg1, arg2) = get_exact_two_args(proc_name, args)?;

    let result = match (eval(arg1, env)?, eval(arg2, env)?) {
        (Expr::Str(lhs), Expr::Str(rhs)) => lhs.cmp(&rhs),
        _ => {
            return Err(format!(
                "{}: both arguments must evaluate to strings.",
                proc_name
            ))
        }
    };

    Ok(Expr::from(result as i32))
}

pub fn concat(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut args = args.iter();
    let mut result = String::from("");
    while let Some(expr) = args.next() {
        match eval(expr, env)? {
            Expr::Str(text) => result += &text,
            _ => {
                return Err(format!(
                    "{}: `{}` does not evaluate to a string.",
                    proc_name, expr
                ))
            }
        }
    }
    Ok(Expr::Str(result))
}

pub fn length(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;
    if let Expr::Str(text) = eval(expr, env)? {
        Ok(Expr::from(text.chars().count() as i32))
    } else {
        Err(format!(
            "{}: `{}` does not evaluate to a string.",
            proc_name, expr
        ))
    }
}

pub fn slice(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (arg1, arg2, arg3) = get_exact_3_args(proc_name, args)?;

    let text = eval_to_str(arg1, env)?;
    let beg = eval_to_num(arg2, env)?;
    let len = eval_to_num(arg3, env)?;

    if beg.fract() != 0.0 || len.fract() != 0.0 {
        return Err(format!(
            "{}: start and end must be integers, but got {} and {}.",
            proc_name, beg, len
        ));
    }

    let beg = beg as usize;
    let len = len as usize;

    Ok(Expr::Str(
        text.chars().skip(beg).take(len).collect::<String>(),
    ))
}
