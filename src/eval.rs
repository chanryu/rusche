use std::rc::Rc;

use crate::env::Env;
use crate::expr::Expr;
use crate::list::List;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Rc<Env>) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(List::Cons(cons)) => {
            if let Expr::Proc(proc) = eval(&cons.car, env)? {
                let args = &cons.cdr;
                proc.invoke(args, env)
            } else {
                Err(format!("{} does not evaluate to a callable.", cons.car))
            }
        }
        _ => Ok(expr.clone()),
    }
}
