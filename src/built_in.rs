pub mod num;
pub mod quote;

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::{cons, List};
use crate::proc::Proc;

const TRUE: Expr = Expr::Num(1_f64);
const FALSE: Expr = NIL;

pub fn atom(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error(func_name, args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error(func_name, args));
    };

    if eval(cons.car.as_ref(), env)?.is_atom() {
        Ok(TRUE)
    } else {
        Ok(FALSE)
    }
}

pub fn car(func_name: &str, args: &List, env: &Env) -> EvalResult {
    if args.len() != 1 {
        return Err(make_syntax_error(func_name, args));
    }

    let mut iter = args.iter();
    let Some(expr) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error(func_name, args))
    }
}

pub fn cdr(func_name: &str, args: &List, env: &Env) -> EvalResult {
    if args.len() != 1 {
        return Err(make_syntax_error(func_name, args));
    }

    let mut iter = args.iter();
    let Some(expr) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error(func_name, args))
    }
}

pub fn cons_(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(car) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    let Some(cdr) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    // TODO: Err if iter.next().is_some()

    let car = eval(car, env)?;
    let Expr::List(cdr) = eval(cdr, env)? else {
        return Err(make_syntax_error(func_name, args));
    };

    Ok(crate::list::cons(car, cdr).into())
}

pub fn cond(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next() {
            None => {
                return Ok(NIL);
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = cons.car.as_ref();
                if !eval(car, env)?.is_nil() {
                    if let Some(expr) = cons.cdar() {
                        return eval(expr, env);
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }

    Err(make_syntax_error(func_name, args))
}

pub fn define(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.set(name, eval(expr, env)?);
                Ok(NIL)
            } else {
                Err(format!(
                    "{func_name}: define expects a expression after symbol"
                ))
            }
        }
        Some(Expr::List(List::Cons(cons))) => {
            let Expr::Sym(name) = cons.car.as_ref() else {
                return Err(format!("{func_name}: expects a list of symbols"));
            };

            let body = iter.into();

            // TODO: check if formal_args is a list of symbols.

            env.set(
                name,
                Expr::Proc(Proc::Closure {
                    name: Some(name.to_string()),
                    formal_args: cons.cdr.as_ref().clone(),
                    body: Box::new(body),
                    outer_env: env.clone(),
                }),
            );
            Ok(NIL)
        }
        _ => Err(make_syntax_error(func_name, args)),
    }
}

pub fn defmacro(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::Sym(macro_name)) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    let Some(Expr::List(List::Cons(formal_args))) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    // TODO: check if formal_args is a list of symbols.

    env.set(
        macro_name,
        Expr::Proc(Proc::Macro {
            name: Some(macro_name.clone()),
            formal_args: List::Cons(formal_args.clone()),
            body: Box::new(iter.into()),
        }),
    );
    Ok(NIL)
}

pub fn defun(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::Sym(lambda_name)) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    let Some(Expr::List(List::Cons(formal_args))) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    // TODO: check if formal_args is a list of symbols.

    env.set(
        lambda_name,
        Expr::Proc(Proc::Closure {
            name: Some(lambda_name.clone()),
            formal_args: List::Cons(formal_args.clone()),
            body: Box::new(iter.into()),
            outer_env: env.clone(),
        }),
    );
    Ok(NIL)
}

pub fn display(_: &str, args: &List, env: &Env) -> EvalResult {
    for (index, expr) in args.iter().enumerate() {
        if index > 0 {
            print!(" ");
        }
        match eval(expr, env)? {
            Expr::Str(text) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    Ok(NIL)
}

pub fn eq(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    if let Some(left) = iter.next() {
        if let Some(right) = iter.next() {
            if iter.next().is_none() {
                return if eval(left, env)? == eval(right, env)? {
                    Ok(TRUE)
                } else {
                    Ok(FALSE)
                };
            }
        }
    }

    Err(make_syntax_error(func_name, args))
}

pub fn eval_(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    let Some(expr) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };
    if iter.next().is_some() {
        return Err(make_syntax_error(func_name, args));
    }

    eval(&eval(expr, env)?, env)
}

pub fn lambda(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::List(List::Cons(formal_args))) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    Ok(Expr::Proc(Proc::Closure {
        name: None,
        formal_args: List::Cons(formal_args.clone()),
        body: Box::new(iter.into()),
        outer_env: env.clone(),
    }))
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
    fn test_define() {
        let env = Env::new();
        // (define name "value")
        let ret = define("", &list!(sym("name"), str("value")), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some(str("value")));
    }

    #[test]
    fn test_eq() {
        let env = Env::new();
        // (eq 1 1) => #t
        assert_ne!(eq("", &list!(num(1), num(1)), &env).unwrap(), NIL);
        // (eq 1 2) => ()
        assert_eq!(eq("", &list!(num(1), num(2)), &env).unwrap(), NIL);
        // (eq "str" "str") => #t
        assert_ne!(eq("", &list!(str("str"), str("str")), &env).unwrap(), NIL);
        // (eq 1 "1") => ()
        assert_eq!(eq("", &list!(num(1), str("1")), &env).unwrap(), NIL);
    }
}
