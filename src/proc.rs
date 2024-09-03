use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::List;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    NativeFunc {
        name: String,
        func: fn(args: &List, env: &Env) -> EvalResult,
    },
    Func {
        name: String,
        formal_args: List,
        body: Box<Expr>,
    },
    Lambda {
        formal_args: List,
        lambda_body: Box<Expr>,
        outer_env: Env,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::NativeFunc { name: _, func } => func(args, env),
            Proc::Func {
                name,
                formal_args,
                body,
            } => eval_func(name, formal_args, body, args, env),
            Proc::Lambda {
                formal_args,
                lambda_body,
                outer_env,
            } => eval_closure(formal_args, lambda_body, outer_env, args, env),
        }
    }
}

fn eval_func(
    func_name: &str,
    formal_args: &List,
    body: &Expr,
    args: &List,
    env: &Env,
) -> EvalResult {
    let func_env = env.derive();
    let mut formal_args = formal_args.iter();
    let mut args = args.iter();

    while let Some(formal_arg) = formal_args.next() {
        let Expr::Sym(name) = formal_arg else {
            return Err(format!("{func_name}: formal arg must be a symbol"));
        };

        let Some(expr) = args.next() else {
            return Err(format!("{func_name}: too few args"));
        };

        func_env.set(name, eval(expr, env)?);
    }

    if args.next().is_some() {
        return Err(format!("{func_name}: too many args"));
    }

    Ok(eval(body, &func_env)?)
}

fn eval_closure(
    formal_args: &List,
    lambda_body: &Expr,
    outer_env: &Env,
    args: &List,
    env: &Env,
) -> EvalResult {
    let lambda_env = outer_env.derive();
    let mut formal_args = formal_args.iter();
    let mut args = args.iter();

    while let Some(formal_arg) = formal_args.next() {
        let Expr::Sym(name) = formal_arg else {
            return Err("lambda: formal arg must be a symbol".into());
        };

        let Some(expr) = args.next() else {
            return Err("lambda: too few args".into());
        };

        lambda_env.set(name, eval(expr, env)?);
    }

    if args.next().is_some() {
        return Err("lambda: too many args".into());
    }

    Ok(eval(lambda_body, &lambda_env)?)
}
