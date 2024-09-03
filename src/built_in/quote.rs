use super::make_syntax_error;
use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

pub fn quote(func_name: &str, args: &List, _env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error(func_name, args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error(func_name, args));
    }

    Ok(cons.car.as_ref().clone())
}

pub fn quasiquote(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let List::Cons(cons) = args else {
        return Err(make_syntax_error(func_name, args));
    };

    if !cons.cdr.is_nil() {
        return Err(make_syntax_error(func_name, args));
    }

    let Expr::List(list) = cons.car.as_ref() else {
        return Ok(cons.car.as_ref().clone());
    };

    let mut exprs = Vec::new();
    let mut iter = list.iter();

    while let Some(expr) = iter.next() {
        let Expr::List(list) = expr else {
            exprs.push(expr.clone());
            continue;
        };

        let List::Cons(cons) = list else {
            exprs.push(List::Nil.into());
            continue;
        };

        let Expr::Sym(name) = cons.car.as_ref() else {
            exprs.push(quasiquote("quasiquote", list, env)?);
            continue;
        };

        match name.as_str() {
            "unquote" => {
                if let Some(cdar) = cons.cdar() {
                    exprs.push(eval(cdar, env)?);
                } else {
                    // TODO: error - malformed unquote, i.e. "(unquote)"
                }
            }
            "unquote-splicing" => {
                if let Some(cdar) = cons.cdar() {
                    if let Expr::List(list) = eval(cdar, env)? {
                        if list.is_nil() {
                            exprs.push(List::Nil.into());
                        } else {
                            exprs.extend(list.iter().map(|expr| expr.clone()));
                        }
                    } else {
                    }
                } else {
                    // TODO: error - malformed unquote, i.e. "(unquote-splicing)"
                }
            }
            _ => {
                exprs.push(quasiquote("quasiquote", list, env)?);
            }
        }
    }

    Ok(exprs.into())
}

pub fn unquote(func_name: &str, _args: &List, _env: &Env) -> EvalResult {
    Err(format!("{func_name} (,) used outside of quasiquote"))
}

pub fn unquote_splicing(func_name: &str, _args: &List, _env: &Env) -> EvalResult {
    Err(format!("{func_name} (,@) used outside of quasiquote"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, sym};
    use crate::list::{cons, list};
    use crate::proc::Proc;

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2)) => (1 2)
        let result = quote("", &list!(list!(num(1), num(2))), &env);
        assert_eq!(result, Ok(list!(num(1), num(2)).into()));
    }

    #[test]
    fn test_quasiquote() {
        let env = Env::new();

        // (quasiquote (0 1 2)) => (0 1 2)
        let result = quasiquote("", &list!(list!(num(0), num(1), num(2))), &env);
        assert_eq!(result, Ok(list!(num(0), num(1), num(2)).into()));
    }

    #[test]
    fn test_quasiquote_unquote() {
        let env = Env::new();
        env.set(
            "+",
            Expr::Proc(Proc::NativeFunc {
                name: "add".to_owned(),
                func: crate::built_in::num::add,
            }),
        );

        // (quasiquote (0 (unquote (+ 1 2)) 4)) => (0 3 4)
        let result = quasiquote(
            "",
            &list!(list!(
                num(0),
                list!(sym("unquote"), list!(sym("+"), num(1), num(2))),
                num(4)
            )),
            &env,
        );
        assert_eq!(result, Ok(list!(num(0), num(3), num(4)).into()));
    }

    #[test]
    fn test_quasiquote_unquote_splicing() {
        let env = Env::new();
        env.set(
            "quote",
            Expr::Proc(Proc::NativeFunc {
                name: "quote".to_owned(),
                func: quote,
            }),
        );

        // (quasiquote (0 (unquote-splicing (quote (1 2 3))) 4)) => (0 1 2 3 4)
        let result = quasiquote(
            "",
            &list!(list!(
                num(0),
                list!(
                    sym("unquote-splicing"),
                    list!(sym("quote"), list!(num(1), num(2), num(3)))
                ),
                num(4)
            )),
            &env,
        );
        assert_eq!(
            result,
            Ok(list!(num(0), num(1), num(2), num(3), num(4)).into())
        );
    }
}
