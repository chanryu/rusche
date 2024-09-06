use super::make_syntax_error;
use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
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

fn quasiquote_expr(func_name: &str, expr: &Expr, env: &Env) -> Result<Vec<Expr>, EvalError> {
    // println!("quasiquote_expr: {}", expr);

    let Expr::List(list) = expr else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name: Option<&str> = if let Expr::Sym(name) = cons.car.as_ref() {
        Some(name.as_str())
    } else {
        None
    };

    let mut exprs = Vec::new();
    match car_name {
        Some("unquote") => {
            if let Some(cdar) = cons.cdar() {
                exprs.push(eval(cdar, env)?);
            } else {
                return Err(make_syntax_error("unquote", &List::Nil));
            }
        }
        Some("unquote-splicing") => {
            if let Some(cdar) = cons.cdar() {
                match eval(cdar, env)? {
                    Expr::List(list) => {
                        // TODO: implement comsuming `into_iter()`
                        exprs.extend(list.iter().map(|e| e.clone()));
                    }
                    _ => {
                        return Err(format!(
                            "unquote-splicing: \"{}\" does not evaluate to a list",
                            cdar
                        ));
                    }
                }
            } else {
                return Err(make_syntax_error("unquote-splicing", &List::Nil));
            }
        }
        _ => {
            let mut v = Vec::new();
            for expr in list.iter() {
                v.extend(quasiquote_expr(func_name, expr, env)?);
            }
            exprs.push(v.into());
        }
    }

    // println!("return:");
    // exprs.iter().for_each(|e| println!("  - {e}"));

    Ok(exprs)
}

pub fn quasiquote(func_name: &str, args: &List, env: &Env) -> EvalResult {
    let mut iter = args.iter();

    let Some(expr) = iter.next() else {
        return Err(make_syntax_error(func_name, args));
    };

    if iter.next().is_some() {
        return Err(make_syntax_error(func_name, args));
    }

    match quasiquote_expr(func_name, expr, env) {
        Ok(mut exprs) => {
            if exprs.len() == 1 {
                Ok(exprs.remove(0))
            } else {
                Err(make_syntax_error(func_name, args))
            }
        }
        Err(err) => Err(err),
    }
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

        env.set("x", num(2));

        // `(0 1 ,x 3) => (0 1 2 3)
        let result = quasiquote(
            "",
            &list!(list!(
                num(0),
                num(1),
                list!(sym("unquote"), sym("x")),
                num(3)
            )),
            &env,
        );
        assert_eq!(result, Ok(list!(num(0), num(1), num(2), num(3)).into()));
    }

    #[test]
    fn test_quasiquote_unquote() {
        let env = Env::new();
        env.set(
            "+",
            Expr::Proc(Proc::Native {
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
            Expr::Proc(Proc::Native {
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
