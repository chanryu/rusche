use crate::expr::Expr;

pub type EvalError = &'static str;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: Expr) -> EvalResult {
    match expr {
        Expr::List(cons) => {
            if let Expr::Proc(func) = eval(cons.car)? {
                func(cons.cdr)
            } else {
                Err("A Proc is expected.")
            }
        }
        Expr::Sym(_) => Ok(Expr::Proc(add)),
        _ => Ok(expr),
    }
}

fn add(expr: Expr) -> EvalResult {
    let _ = expr;
    Ok(Expr::Nil)
}
