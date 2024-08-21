use crate::env::Env;
use crate::expr::Expr;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Sym(text) => match env.get(text) {
            Some(Expr::Proc(func)) => Ok(Expr::Proc(func.clone())),
            Some(_) => Err(format!("{} is not a procedure!", text)),
            None => Err(format!("Undefined symbol: {:?}", text)),
        },
        Expr::List(cons) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(&cons.cdr, env)
            } else {
                Err(String::from("A Proc is expected."))
            }
        }
        _ => Ok(expr.clone()),
    }
}
