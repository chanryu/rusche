use std::hash::{DefaultHasher, Hash, Hasher};

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::List;

pub type NativeFunc = fn(func_name: &str, args: &List, env: &Env) -> EvalResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    Closure {
        name: Option<String>,
        formal_args: List,
        body: Box<List>,
        outer_env: Env,
    },
    Macro {
        name: Option<String>,
        formal_args: List,
        body: Box<List>,
    },
    Native {
        name: String,
        func: NativeFunc,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::Closure {
                name,
                formal_args,
                body,
                outer_env,
            } => eval_closure(name.as_deref(), formal_args, body, outer_env, args, env),
            Proc::Macro {
                name,
                formal_args,
                body,
            } => eval_macro(name.as_deref(), formal_args, body, args, env),
            Proc::Native { name, func } => func(name, args, env),
        }
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        match self {
            Proc::Closure {
                name,
                formal_args,
                body,
                outer_env: _,
            } => {
                formal_args.to_string().hash(&mut hasher);
                body.to_string().hash(&mut hasher);
                format!(
                    "proc/closure:{}:{:x}",
                    name.as_deref().unwrap_or("unnamed"),
                    hasher.finish()
                )
            }
            Proc::Macro {
                name,
                formal_args,
                body,
            } => {
                formal_args.to_string().hash(&mut hasher);
                body.to_string().hash(&mut hasher);
                format!(
                    "proc/macro:{}:{:x}",
                    name.as_deref().unwrap_or("unnamed"),
                    hasher.finish()
                )
            }
            Proc::Native { name, func } => {
                func.hash(&mut hasher);
                format!("proc/native:{}:{:x}", name, hasher.finish())
            }
        }
    }
}

fn eval_closure(
    closure_name: Option<&str>,
    formal_args: &List,
    body: &List,
    outer_env: &Env,
    actual_args: &List,
    env: &Env,
) -> EvalResult {
    let closure_name = closure_name.unwrap_or("closure");
    let closure_env = outer_env.derive();
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            let Expr::Sym(arg_name) = formal_arg else {
                return Err(format!("{}: formal arg must be a symbol", closure_name));
            };

            if let Some(name) = parse_name_if_variadic_args(arg_name) {
                closure_env.set(name, actual_args);
                break;
            }

            let Some(expr) = actual_args.next() else {
                return Err(format!("{}: too few args", closure_name));
            };

            closure_env.set(arg_name, eval(expr, env)?);
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(format!("{}: too many args", closure_name));
        }
    }

    let result = body
        .iter()
        .try_fold(NIL, |_, expr| eval(expr, &closure_env))?;
    Ok(result)
}

fn eval_macro(
    macro_name: Option<&str>,
    formal_args: &List,
    body: &List,
    actual_args: &List,
    env: &Env,
) -> EvalResult {
    let macro_name = macro_name.unwrap_or("macro");
    let macro_env = env.derive();
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            let Expr::Sym(arg_name) = formal_arg else {
                return Err(format!("{}: formal arg must be a symbol", macro_name));
            };

            if let Some(name) = parse_name_if_variadic_args(arg_name) {
                macro_env.set(name, actual_args);
                break;
            }

            let Some(expr) = actual_args.next() else {
                return Err(format!("{}: too few args", macro_name));
            };

            macro_env.set(arg_name, expr.clone());
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(format!("{}: too many args", macro_name));
        }
    }

    let mut result = NIL;
    for expr in body.iter() {
        let expanded = eval(expr, &macro_env)?;
        result = eval(&expanded, env)?;
    }
    Ok(result)
}

fn parse_name_if_variadic_args(name: &str) -> Option<&str> {
    if name.starts_with("*") && name.len() > 1 {
        Some(&name[1..])
    } else {
        None
    }
}
