pub mod num;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::{Expr, ExprIter, NIL};

pub fn define(args: &Expr, env: &Env) -> EvalResult {
    let mut args = ExprIter::new(args);

    match args.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = args.next() {
                env.set(name, eval(expr, env)?.clone());
                Ok(NIL)
            } else {
                Err("define expects a expression after symbol".to_string())
            }
        }
        _ => Err("define expects a symbol".to_string()),
    }
}

pub fn quote(args: &Expr, _env: &Env) -> EvalResult {
    let mut args = ExprIter::new(args);

    match args.next() {
        Some(Expr::List(Some(cons))) => Ok(cons.car.as_ref().clone()),
        _ => Err("quote requires an expression.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define() {
        let env = Env::new();
        let ret = define(
            &Expr::new_list(
                Expr::new_sym("name"),
                Expr::new_list(Expr::new_str("value"), NIL),
            ),
            &env,
        );
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.get("name"), Some(Expr::new_str("value")));
    }
}
