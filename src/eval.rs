use crate::env::Env;
use crate::expr::Expr;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.get(name) {
            Some(Expr::Proc(func)) => Ok(Expr::Proc(func.clone())),
            Some(_) => Err(format!("{} is not a procedure!", name)),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(Some(cons)) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(&cons.cdr, env)
            } else {
                Err(String::from("A Proc is expected."))
            }
        }
        _ => Ok(expr.clone()),
    }
}
