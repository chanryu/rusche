pub mod num;

use crate::eval::{eval, Env, EvalError, EvalResult};
use crate::expr::{Cons, Expr, NIL};

pub fn define(args: &Expr, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
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
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        Ok(car.clone())
    } else {
        Err(make_syntax_error("quote", args))
    }
}

pub fn atom(args: &Expr, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        match eval(car, env)? {
            Expr::List(Some(_)) => Ok(NIL),
            _ => Ok(Expr::Sym(String::from("#t"))),
        }
    } else {
        Err(make_syntax_error("atom", args))
    }
}

pub fn car(args: &Expr, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        Ok(eval(car, env)?)
    } else {
        Err(make_syntax_error("car", args))
    }
}

pub fn cdr(args: &Expr, env: &Env) -> EvalResult {
    if let Some(cdr) = args.cdr() {
        if let Some(car) = cdr.car() {
            return Ok(eval(car, env)?);
        }
    }

    Err(make_syntax_error("cdr", args))
}

fn make_syntax_error(func_name: &str, args: &Expr) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        Expr::List(Some(Cons::new(
            Expr::Sym(func_name.to_string()),
            args.clone(),
        )))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::*;

    #[test]
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define(&cons(sym("name"), cons(str("value"), NIL)), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.get("name"), Some(str("value")));
    }

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2))
        let ret = quote(&cons(cons(num(1), cons(num(2), NIL)), NIL), &env);
        assert_eq!(ret, Ok(cons(num(1), cons(num(2), NIL))));
    }

    #[test]
    fn test_car() {
        let env = Env::new();
        // (car '(1 2)) => 1
        let ret = car(&cons(num(1), cons(num(2), NIL)), &env);
        assert_eq!(ret, Ok(num(1)));
    }

    #[test]
    fn test_cdr() {
        let env = Env::new();
        // (cdr '(1 2)) => 2
        let ret = cdr(&cons(num(1), cons(num(2), NIL)), &env);
        assert_eq!(ret, Ok(num(2)));
    }
}
