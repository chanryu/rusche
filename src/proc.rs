use std::rc::Rc;

use crate::eval::{Env, EvalResult};
use crate::list::List;

#[derive(Clone, Debug, PartialEq)]
pub enum Proc {
    Native(NativeFunc),
    Lambda {
        formal_args: List,
        outer_env: Rc<Env>,
        lambda_func: LambdaFunc,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, env: &Env) -> EvalResult {
        match self {
            Proc::Native(func) => func(args, env),
            Proc::Lambda {
                formal_args,
                outer_env,
                lambda_func,
            } => lambda_func(formal_args, outer_env, args, env),
        }
    }
}

pub type NativeFunc = fn(args: &List, env: &Env) -> EvalResult;
pub type LambdaFunc = fn(formal_args: &List, outer_env: &Env, args: &List, env: &Env) -> EvalResult;
