use super::{get_exact_one_arg, make_syntax_error};
use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::List;

pub fn quote(proc_name: &str, args: &List, _env: &Env) -> EvalResult {
    Ok(get_exact_one_arg(proc_name, args)?.clone())
}

fn quasiquote_expr(proc_name: &str, expr: &Expr, env: &Env) -> Result<Vec<Expr>, EvalError> {
    let Expr::List(list) = expr else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name = if let Expr::Sym(name) = cons.car.as_ref() {
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
                v.extend(quasiquote_expr(proc_name, expr, env)?);
            }
            exprs.push(v.into());
        }
    }

    Ok(exprs)
}

pub fn quasiquote(proc_name: &str, args: &List, env: &Env) -> EvalResult {
    let expr = get_exact_one_arg(proc_name, args)?;

    match quasiquote_expr(proc_name, expr, env) {
        Ok(mut exprs) => {
            if exprs.len() == 1 {
                Ok(exprs.remove(0))
            } else {
                Err(make_syntax_error(proc_name, args))
            }
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, sym};
    use crate::macros::list;
    use crate::proc::Proc;

    #[test]
    fn test_quote() {
        let env = Env::new();
        // (quote (1 2)) => (1 2)
        let result = quote("", &list!(list!(1, 2)), &env);
        assert_eq!(result, Ok(list!(1, 2).into()));
    }

    #[test]
    fn test_quasiquote() {
        let env = Env::new();

        env.set("x", num(2));

        // `(0 1 ,x 3) => (0 1 2 3)
        let result = quasiquote(
            "",
            &list!(list!(0, 1, list!(sym("unquote"), sym("x")), 3)),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3).into()));
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
            &list!(list!(0, list!(sym("unquote"), list!(sym("+"), 1, 2)), 4)),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 3, 4).into()));
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
                0,
                list!(sym("unquote-splicing"), list!(sym("quote"), list!(1, 2, 3))),
                4
            )),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3, 4).into()));
    }
}
