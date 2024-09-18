mod num;
mod quote;
mod str;
mod utils;

use std::rc::Rc;

use utils::{get_exact_1_arg, get_exact_2_args, make_formal_args};

use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{intern, Expr, NIL};
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

    // lisp primitives
    set_native_func("atom?", atom);
    set_native_func("car", car);
    set_native_func("cdr", cdr);
    set_native_func("cons", cons_);
    set_native_func("cond", cond);
    set_native_func("define", define);
    set_native_func("defmacro", defmacro);
    set_native_func("eq?", eq);
    set_native_func("eval", eval_);
    set_native_func("lambda", lambda);
    set_native_func("set!", set);

    // quote
    set_native_func("quote", quote::quote);
    set_native_func("quasiquote", quote::quasiquote);

    // num
    set_native_func("num?", num::is_num);
    set_native_func("+", num::add);
    set_native_func("-", num::subtract);
    set_native_func("*", num::multiply);
    set_native_func("/", num::divide);

    // str
    set_native_func("str?", str::is_str);
    set_native_func("str-compare", str::compare);
    set_native_func("str-concat", str::concat);
    set_native_func("str-length", str::length);
    set_native_func("str-slice", str::slice);
}

pub fn atom(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    Ok(eval(expr, env)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cdr(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons)) = eval(expr, env)? {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(make_syntax_error(proc_name, args))
    }
}

pub fn cons_(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (car, cdr) = get_exact_2_args(proc_name, args)?;

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
    let (left, right) = get_exact_2_args(proc_name, args)?;

    Ok((eval(left, env)? == eval(right, env)?).into())
}

pub fn eval_(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

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
    let (name_expr, value_expr) = get_exact_2_args(proc_name, args)?;

    let Expr::Sym(name) = name_expr else {
        return Err("".to_owned());
    };

    env.update(name, eval(value_expr, &env)?);

    Ok(NIL)
}

fn make_syntax_error(proc_name: &str, args: &List) -> EvalError {
    format!(
        "Ill-formed syntax: {}",
        cons(intern(proc_name), args.clone())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::list;

    #[test]
    fn test_define() {
        let env = Env::for_unit_test();

        // (define name "value")
        let ret = define("", &list!(intern("name"), "value"), &env);
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
