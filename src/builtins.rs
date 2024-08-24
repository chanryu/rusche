pub mod num;

use crate::eval::{eval, Env, EvalResult};
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
    match args {
        Expr::List(Some(cons)) => Ok((*cons.car).clone()),
        _ => Err("quote requires an expression.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::*;

    #[test]
    fn test_define() {
        let env = Env::new();
        let ret = define(&cons(sym("name"), cons(str("value"), NIL)), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.get("name"), Some(str("value")));
    }
}
