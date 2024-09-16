pub mod num;
pub mod quote;

use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::{cons, List};
use crate::proc::Proc;

pub fn load_builtin(env: &Rc<Env>) {
    let set_native_func = |name, func| {
        env.define(
            name,
            Expr::Proc(Proc::Native {
                name: name.to_owned(),
                func,
            }),
        );
    };

    use crate::built_in;

    // lisp primitives
    set_native_func("atom?", built_in::atom);
    set_native_func("car", built_in::car);
    set_native_func("cdr", built_in::cdr);
    set_native_func("cons", built_in::cons_);
    set_native_func("cond", built_in::cond);
    set_native_func("define", built_in::define);
    set_native_func("defmacro", built_in::defmacro);
    set_native_func("eq?", built_in::eq);
    set_native_func("eval", built_in::eval_);
    set_native_func("lambda", built_in::lambda);
    set_native_func("set!", built_in::set);

    // quote
    set_native_func("quote", built_in::quote::quote);
    set_native_func("quasiquote", built_in::quote::quasiquote);

    // num
    set_native_func("+", built_in::num::add);
    set_native_func("-", built_in::num::minus);
    set_native_func("*", built_in::num::multiply);
    set_native_func("/", built_in::num::divide);
    set_native_func("num?", built_in::num::is_num);
}

pub fn atom(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    Ok(eval(expr, env)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cdr(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cons_(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (car, cdr) = get_exact_two_args(proc_name, args)?;

    let car = eval(car, env)?;
    let Expr::List(cdr) = eval(cdr, env)? else {
        return Err(format!("{proc_name}: {cdr} does not evaluate to a list."));
    };

    Ok(cons(car, cdr).into())
}

pub fn cond(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();
    loop {
        match iter.next() {
            None => {
                return Ok(NIL);
            }
            Some(Expr::List(List::Cons(cons))) => {
                let car = &cons.car;
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

pub fn define(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name)) => {
            if let Some(expr) = iter.next() {
                env.define(name, eval(expr, env)?);
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

            let formal_args = make_formal_args(&cons.cdr)?;

            env.define(
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

pub fn defmacro(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut iter = args.iter();

    let (macro_name, formal_args) = match iter.next() {
        // (defmacro name (args) body)
        Some(Expr::Sym(macro_name)) => {
            let Some(Expr::List(list)) = iter.next() else {
                return Err(make_syntax_error(proc_name, args));
            };

            (macro_name, make_formal_args(list)?)
        }
        // (defmacro (name args) body)
        Some(Expr::List(List::Cons(cons))) => {
            let Expr::Sym(macro_name) = cons.car.as_ref() else {
                return Err(make_syntax_error(proc_name, args));
            };

            (macro_name, make_formal_args(&cons.cdr)?)
        }
        _ => return Err(make_syntax_error(proc_name, args)),
    };

    env.define(
        macro_name,
        Expr::Proc(Proc::Macro {
            name: Some(macro_name.clone()),
            formal_args: formal_args,
            body: Box::new(iter.into()),
        }),
    );

    Ok(NIL)
}

pub fn eq(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (left, right) = get_exact_two_args(proc_name, args)?;

    Ok((eval(left, env)? == eval(right, env)?).into())
}

pub fn eval_(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    eval(&eval(expr, env)?, env)
}

pub fn lambda(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
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

pub fn set(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (name_expr, value_expr) = get_exact_two_args(proc_name, args)?;

    let Expr::Sym(name) = name_expr else {
        return Err("".to_owned());
    };

    env.update(name, eval(value_expr, &env)?);

    Ok(NIL)
}

fn make_syntax_error(proc_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(Expr::Sym(proc_name.into()), args.clone())
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
    use crate::expr::shortcuts::sym;
    use crate::macros::list;

    #[test]
    fn test_define() {
        let env = Env::for_unit_test();

        // (define name "value")
        let ret = define("", &list!(sym("name"), "value"), &env);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some("value".into()));
    }

    #[test]
    fn test_eq() {
        let env = Env::for_unit_test();

        // (eq 1 1) => #t
        assert_ne!(eq("", &list!(1, 1), &env).unwrap(), NIL);
        // (eq 1 2) => ()
        assert_eq!(eq("", &list!(1, 2), &env).unwrap(), NIL);
        // (eq "str" "str") => #t
        assert_ne!(eq("", &list!("str", "str"), &env).unwrap(), NIL);
        // (eq 1 "1") => ()
        assert_eq!(eq("", &list!(1, "1"), &env).unwrap(), NIL);
    }
}
