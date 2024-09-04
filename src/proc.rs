use std::hash::{DefaultHasher, Hash, Hasher};

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    Lambda {
        name: Option<String>,
        formal_args: List,
        body: Box<Expr>,
        outer_env: Env,
    },
    Macro {
        name: Option<String>,
        formal_args: List,
        body: Box<Expr>,
    },
    Native {
        name: String,
        func: fn(func_name: &str, args: &List, env: &Env) -> EvalResult,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::Lambda {
                name,
                formal_args,
                body,
                outer_env,
            } => eval_lambda(name.as_deref(), formal_args, body, outer_env, args, env),
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
            Proc::Lambda {
                name,
                formal_args,
                body,
                outer_env: _,
            } => {
                formal_args.to_string().hash(&mut hasher);
                body.to_string().hash(&mut hasher);
                format!(
                    "proc/lambda:{}:{:x}",
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

fn eval_lambda(
    lambda_name: Option<&str>,
    formal_args: &List,
    body: &Expr,
    outer_env: &Env,
    actual_args: &List,
    env: &Env,
) -> EvalResult {
    let lambda_env = outer_env.derive();
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    while let Some(formal_arg) = formal_args.next() {
        let Expr::Sym(name) = formal_arg else {
            return Err(format!(
                "{}: formal arg must be a symbol",
                lambda_name.unwrap_or("lambda")
            ));
        };

        let Some(expr) = actual_args.next() else {
            return Err(format!("{}: too few args", lambda_name.unwrap_or("lambda")));
        };

        lambda_env.set(name, eval(expr, env)?);
    }

    if actual_args.next().is_some() {
        return Err(format!(
            "{}: too many args",
            lambda_name.unwrap_or("lambda")
        ));
    }

    Ok(eval(body, &lambda_env)?)
}

fn eval_macro(
    macro_name: Option<&str>,
    formal_args: &List,
    body: &Expr,
    actual_args: &List,
    env: &Env,
) -> EvalResult {
    let macro_env = env.derive();
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    while let Some(formal_arg) = formal_args.next() {
        let Expr::Sym(name) = formal_arg else {
            return Err(format!(
                "{}: formal arg must be a symbol",
                macro_name.unwrap_or("macro")
            ));
        };

        let Some(expr) = actual_args.next() else {
            return Err(format!("{}: too few args", macro_name.unwrap_or("macro")));
        };

        macro_env.set(name, expr.clone());
    }

    if actual_args.next().is_some() {
        return Err(format!("{}: too many args", macro_name.unwrap_or("macro")));
    }

    Ok(eval(body, &macro_env)?)
}
