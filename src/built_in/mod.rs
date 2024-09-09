pub mod num;
pub mod quote;

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::{cons, List};
use crate::proc::Proc;

pub fn atom(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    Ok(eval(expr, env)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cdr(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cons_(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let (car, cdr) = get_exact_two_args(proc_name, args)?;

    let car = eval(car, env)?;
    let Expr::List(cdr) = eval(cdr, env)? else {
        return Err(format!("{proc_name}: {cdr} does not evaluate to a list."));
    };

    Ok(cons(car, cdr).into())
}

pub fn cond(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next() {
            None => {
                return Ok(NIL);
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = cons.car.as_ref();
                if eval(car, env)?.is_truthy() {
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

    Err(make_syntax_error(proc_name, args))
}

pub fn define(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.set(name, eval(expr, env)?);
                Ok(NIL)
            } else {
                Err(format!(
                    "{proc_name}: define expects a expression after symbol"
                ))
            }
        }
        Some(Expr::List(List::Cons(cons))) => {
            let Expr::Sym(name) = cons.car.as_ref() else {
                return Err(format!("{proc_name}: expects a list of symbols"));
            };

            let formal_args = make_formal_args(cons.cdr.as_ref())?;

            env.set(
                name,
                Expr::Proc(Proc::Closure {
                    name: Some(name.to_string()),
                    formal_args: formal_args.clone(),
                    body: Box::new(iter.into()),
                    outer_env: env.clone(),
                }),
            );
            Ok(NIL)
        }
        _ => Err(make_syntax_error(proc_name, args)),
    }
}

pub fn defmacro(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::Sym(macro_name)) = iter.next() else {
        return Err(make_syntax_error(proc_name, args));
    };

    let Some(Expr::List(list)) = iter.next() else {
        return Err(make_syntax_error(proc_name, args));
    };

    env.set(
        macro_name,
        Expr::Proc(Proc::Macro {
            name: Some(macro_name.clone()),
            formal_args: make_formal_args(list)?,
            body: Box::new(iter.into()),
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

pub fn eq(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let (left, right) = get_exact_two_args(proc_name, args)?;

    Ok((eval(left, env)? == eval(right, env)?).into())
}

pub fn eval_(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    eval(&eval(expr, env)?, env)
}

pub fn lambda(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(Expr::List(list)) = iter.next() else {
        return Err(make_syntax_error(proc_name, args));
    };

    Ok(Expr::Proc(Proc::Closure {
        name: None,
        formal_args: make_formal_args(list)?,
        body: Box::new(iter.into()),
        outer_env: env.clone(),
    }))
}

fn make_syntax_error(proc_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(Expr::new_sym(proc_name), args.clone())
    )
}

fn get_exact_one_arg<'a>(proc_name: &str, args: &'a List) -> Result<&'a Expr, EvalError> {
    let mut iter = args.iter();
    let Some(arg) = iter.next() else {
        return Err(format!("{proc_name} needs an argument."));
    };
    if iter.next().is_none() {
        Ok(arg)
    } else {
        Err(format!("{proc_name} expects only 1 argument."))
    }
}

fn get_exact_two_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();
    let Some(arg0) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    let Some(arg1) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    if iter.next().is_none() {
        Ok((arg0, arg1))
    } else {
        Err(format!("{}: takes only two arguments", proc_name))
    }
}

fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg) = item else {
            return Err(format!("{item} is not a symbol."));
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
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
