pub mod num;

use crate::eval::{eval, Env, EvalError, EvalResult};
use crate::expr::{sym, Expr, IntoExpr};
use crate::list::List;

pub fn atom(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        if eval(car, env)?.is_atom() {
            Ok(sym("#t"))
        } else {
            Ok(List::Nil.into_expr())
        }
    } else {
        Err(make_syntax_error("atom", args))
    }
}

pub fn car(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        Ok(eval(car, env)?)
    } else {
        Err(make_syntax_error("car", args))
    }
}

pub fn cdr(args: &List, env: &Env) -> EvalResult {
    if let Some(cdr) = args.cdr() {
        if let Some(cdar) = cdr.car() {
            return Ok(eval(cdar, env)?);
        }
    }

    Err(make_syntax_error("cdr", args))
}

pub fn cond(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next() {
            None => {
                return Ok(List::Nil.into_expr());
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = cons.car.as_ref();
                if eval(car, env)? != List::Nil.into_expr() {
                    if let Some(cdar) = cons.cdr.car() {
                        return eval(cdar, env);
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }

    Err(make_syntax_error("cond", args))
}

pub fn quote(args: &List, _env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        Ok(car.clone())
    } else {
        Err(make_syntax_error("quote", args))
    }
}

pub fn define(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.set(name, eval(expr, env)?.clone());
                Ok(List::Nil.into_expr())
            } else {
                Err("define expects a expression after symbol".to_string())
            }
        }
        _ => Err("define expects a symbol".to_string()),
    }
}

pub fn eq(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        if let Some(cdr) = args.cdr() {
            if let Some(cdar) = cdr.car() {
                let arg1 = eval(car, env)?;
                let arg2 = eval(cdar, env)?;
                return if arg1 == arg2 {
                    Ok(sym("#t"))
                } else {
                    Ok(List::Nil.into_expr())
                };
            }
        }
    }

    Err(make_syntax_error("eq", args))
}

fn make_syntax_error(func_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        Expr::List(List::new_cons(
            Expr::Sym(func_name.to_string()),
            args.clone()
        ))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{num, str, sym, IntoExpr};
    use crate::list::cons;

    #[test]
    fn test_car() {
        let env = Env::new();
        // (car '(1 2)) => 1
        let ret = car(&cons(num(1), cons(num(2), List::Nil)), &env);
        assert_eq!(ret, Ok(num(1)));
    }

    #[test]
    fn test_cdr() {
        let env = Env::new();
        // (cdr '(1 2)) => 2
        let ret = cdr(&cons(num(1), cons(num(2), List::Nil)), &env);
        assert_eq!(ret, Ok(num(2)));
    }

    #[test]
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define(&cons(sym("name"), cons(str("value"), List::Nil)), &env);
        assert_eq!(ret, Ok(List::Nil.into_expr()));
        assert_eq!(env.get("name"), Some(str("value")));
    }

    #[test]
    fn test_eq() {
        let env = Env::new();
        // (eq 1 1) => #t
        assert_eq!(
            eq(&cons(num(1), cons(num(1), List::Nil)), &env),
            Ok(sym("#t"))
        );
        // (eq 1 2) => ()
        assert_eq!(
            eq(&cons(num(1), cons(num(2), List::Nil)), &env),
            Ok(List::Nil.into_expr())
        );
        // (eq "str" "str") => #t
        assert_eq!(
            eq(&cons(str("str"), cons(str("str"), List::Nil)), &env),
            Ok(sym("#t"))
        );
        // (eq 1 "1") => ()
        assert_eq!(
            eq(&cons(num(1), cons(str("1"), List::Nil)), &env),
            Ok(List::Nil.into_expr())
        );
    }

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2))
        let ret = quote(
            &cons(cons(num(1), cons(num(2), List::Nil)), List::Nil),
            &env,
        );
        assert_eq!(ret, Ok(cons(num(1), cons(num(2), List::Nil)).into_expr()));
    }
}
