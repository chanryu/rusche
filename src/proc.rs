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
        body: List,
    },
    Lambda {
        formal_args: List,
        lambda_body: List,
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
    _name: &str,
    _formal_args: &List,
    _body: &List,
    _args: &List,
    _env: &Env,
) -> EvalResult {
    todo!()
}

fn eval_closure(
    formal_args: &List,
    lambda_body: &List,
    outer_env: &Env,
    args: &List,
    env: &Env,
) -> EvalResult {
    let lambda_env = outer_env.derive();
    let mut formal_args = formal_args.iter();
    let mut args = args.iter();

    while let Some(formal_arg) = formal_args.next() {
        if let Expr::Sym(name) = formal_arg {
            if let Some(expr) = args.next() {
                lambda_env.set(name, eval(expr, env)?);
            } else {
                return Err("Proc: too few args".into());
            }
        } else {
            return Err("Formal arg of lambda must be a symbol".into());
        }
    }
    if args.next() != None {
        return Err("Proc: too many args".into());
    }

    let mut result = NIL;
    for expr in lambda_body.iter() {
        result = eval(expr, &lambda_env)?;
    }
    Ok(result)
}
