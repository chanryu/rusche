use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    Native {
        name: String,
        func: fn(func_name: &str, args: &List, env: &Env) -> EvalResult,
    },
    Lambda {
        name: Option<String>,
        formal_args: List,
        body: Box<Expr>,
        outer_env: Env,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::Native { name, func } => func(name, args, env),
            Proc::Lambda {
                name,
                formal_args,
                body,
                outer_env,
            } => eval_lambda(name.as_deref(), formal_args, body, outer_env, args, env),
        }
    }
}

fn eval_lambda(
    lambda_name: Option<&str>,
    formal_args: &List,
    body: &Expr,
    outer_env: &Env,
    args: &List,
    env: &Env,
) -> EvalResult {
    let lambda_env = outer_env.derive();
    let mut formal_args = formal_args.iter();
    let mut args = args.iter();

    while let Some(formal_arg) = formal_args.next() {
        let Expr::Sym(name) = formal_arg else {
            return Err(format!(
                "{}: formal arg must be a symbol",
                lambda_name.unwrap_or("lambda")
            ));
        };

        let Some(expr) = args.next() else {
            return Err(format!("{}: too few args", lambda_name.unwrap_or("lambda")));
        };

        lambda_env.set(name, eval(expr, env)?);
    }

    if args.next().is_some() {
        return Err(format!(
            "{}: too many args",
            lambda_name.unwrap_or("lambda")
        ));
    }

    Ok(eval(body, &lambda_env)?)
}
