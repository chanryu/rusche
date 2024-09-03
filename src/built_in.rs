pub mod num;
pub mod quote;

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::{cons, List};
use crate::proc::Proc;

pub fn atom(args: &List, env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error("atom", args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error("atom", args));
    };

    if eval(cons.car.as_ref(), env)?.is_atom() {
        Ok(Expr::new_sym("#t"))
    } else {
        Ok(NIL)
    }
}

pub fn car(args: &List, _env: &Env) -> EvalResult {
    if let List::Cons(cons) = args {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error("car", args))
    }
}

pub fn cdr(args: &List, _env: &Env) -> EvalResult {
    if let List::Cons(cons) = args {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error("cdr", args))
    }
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
                if !eval(car, env)?.is_nil() {
                    if let List::Cons(cons) = cons.cdr.as_ref() {
                        return eval(cons.car.as_ref(), env);
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

pub fn define(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.set(name, eval(expr, env)?);
                Ok(NIL)
            } else {
                Err("define expects a expression after symbol".into())
            }
        }
        Some(Expr::List(List::Cons(cons))) => {
            let Expr::Sym(name) = cons.car.as_ref() else {
                return Err("define expects a list of symbols".into());
            };

            let Some(body) = iter.next() else {
                return Err("define expects a expression after a list of symbols".into());
            };

            // TODO: check if formal_args is a list of symbols.

            env.set(
                name,
                Expr::Proc(Proc::Func {
                    name: name.clone(),
                    formal_args: cons.cdr.as_ref().clone(),
                    body: Box::new(body.clone()),
                }),
            );
            Ok(NIL)
        }
        _ => Err("define expects a symbol".into()),
    }
}

pub fn eq(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    if let Some(left) = iter.next() {
        if let Some(right) = iter.next() {
            if iter.next().is_none() {
                return if eval(left, env)? == eval(right, env)? {
                    Ok(Expr::new_sym("#t"))
                } else {
                    Ok(NIL)
                };
            }
        }
    }

    Err(make_syntax_error("eq", args))
}

pub fn lambda(args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::List(List::Cons(formal_args))) = iter.next() else {
        return Err(make_syntax_error("lambda", args));
    };

    // TODO: check if formal_args is a list of symbols.

    let Some(lambda_body) = iter.next() else {
        return Err(make_syntax_error("lambda", args));
    };

    if iter.next().is_some() {
        return Err(make_syntax_error("lambda", args));
    }

    return Ok(Expr::Proc(Proc::Lambda {
        formal_args: List::Cons(formal_args.clone()),
        lambda_body: Box::new(lambda_body.clone()),
        outer_env: env.clone(),
    }));
}

fn make_syntax_error(func_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(Expr::new_sym(func_name), args.clone())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, str, sym};
    use crate::list::list;

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
        assert_eq!(ret, Ok(list!(num(2)).into()));
    }

    #[test]
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define(&list!(sym("name"), str("value")), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some(str("value")));
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
}
