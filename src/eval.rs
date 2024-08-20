use crate::env::Env;
use crate::expr::Expr;

pub type EvalResult = Result<Expr, String>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Nil => Ok(Expr::Nil),
        Expr::Num(value) => Ok(Expr::Num(value.clone())),
        Expr::Str(text) => Ok(Expr::Str(text.clone())),
        Expr::Sym(text) => match env.get(text) {
            Some(Expr::Proc(func)) => Ok(Expr::Proc(func.clone())),
            Some(_) => Err(format!("{} is not a procedure!", text)),
            None => Err(format!("Undefined symbol: {:?}", text)),
        },
        Expr::Proc(func) => Ok(Expr::Proc(func.clone())),
        Expr::List(cons) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(&cons.cdr, env)
            } else {
                Err(String::from("A Proc is expected."))
            }
        }
    }
}
