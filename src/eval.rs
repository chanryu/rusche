use crate::env::Env;
use crate::expr::{Expr, NIL};
use crate::list::List;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(list) => eval_list(list, env),
        _ => Ok(expr.clone()),
    }
}

pub fn eval_list(list: &List, env: &Env) -> EvalResult {
    match list {
        List::Nil => Ok(NIL),
        List::Cons(cons) => {
            if let Expr::Proc(proc) = eval(&cons.car, env)? {
                proc.invoke(cons.cdr.as_ref(), env)
            } else {
                Err(format!("{} does not evaluate to a callable.", cons.car))
            }
        }
    }
}
