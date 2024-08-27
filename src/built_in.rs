pub mod num;

use crate::eval::{eval, Env, EvalError, EvalResult};
use crate::expr::{sym, Expr, NIL};
use crate::list::{cons, List};

pub fn atom(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        // TODO: error if cdr is not NIL
        if eval(car, env)?.is_atom() {
            Ok(sym("#t"))
        } else {
            Ok(NIL)
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
                return Ok(NIL);
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = cons.car.as_ref();
                if eval(car, env)? != NIL {
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
                Ok(NIL)
            } else {
                Err("define expects a expression after symbol".into())
            }
        }
        _ => Err("define expects a symbol".into()),
    }
}

pub fn eq(args: &List, env: &Env) -> EvalResult {
    if let Some(car) = args.car() {
        if let Some(cdr) = args.cdr() {
            if let Some(cdar) = cdr.car() {
                let arg1 = eval(car, env)?;
                let arg2 = eval(cdar, env)?;
                return if arg1 == arg2 { Ok(sym("#t")) } else { Ok(NIL) };
            }
        }
    }

    Err(make_syntax_error("eq", args))
}

fn make_syntax_error(func_name: &str, args: &List) -> EvalError {
    format!("Ill-formed syntax: {}", cons(sym(func_name), args.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{num, str, sym};
    use crate::list::cons;
    use crate::macros::list;

    #[test]
    fn test_car() {
        let env = Env::new();
        // (car '(1 2)) => 1
        let ret = car(&list!(num(1), num(2)), &env);
        assert_eq!(ret, Ok(num(1)));
    }

    #[test]
    fn test_cdr() {
        let env = Env::new();
        // (cdr '(1 2)) => 2
        let ret = cdr(&list!(num(1), num(2)), &env);
        assert_eq!(ret, Ok(num(2)));
    }

    #[test]
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define(&list!(sym("name"), str("value")), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.get("name"), Some(str("value")));
    }

    #[test]
    fn test_eq() {
        let env = Env::new();
        // (eq 1 1) => #t
        assert_eq!(eq(&list!(num(1), num(1)), &env), Ok(sym("#t")));
        // (eq 1 2) => ()
        assert_eq!(eq(&list!(num(1), num(2)), &env), Ok(NIL));
        // (eq "str" "str") => #t
        assert_eq!(eq(&list!(str("str"), str("str")), &env), Ok(sym("#t")));
        // (eq 1 "1") => ()
        assert_eq!(eq(&list!(num(1), str("1")), &env), Ok(NIL));
    }

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2)) => (1 2)
        let ret = quote(&list!(list!(num(1), num(2))), &env);
        assert_eq!(ret, Ok(list!(num(1), num(2)).into()));
    }
}
