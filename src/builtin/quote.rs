use std::rc::Rc;

use super::{get_exact_1_arg, make_syntax_error};
use crate::env::Env;
use crate::eval::{eval, EvalError, EvalResult};
use crate::expr::{Expr, ExprKind, NIL};
use crate::list::List;

pub fn quote(proc_name: &str, args: &List, _env: &Rc<Env>) -> EvalResult {
    Ok(get_exact_1_arg(proc_name, args)?.clone())
}

pub fn quasiquote(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

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

fn quasiquote_expr(proc_name: &str, expr: &Expr, env: &Rc<Env>) -> Result<Vec<Expr>, EvalError> {
    let ExprKind::List(list) = &expr.kind else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name = if let ExprKind::Sym(name) = &cons.car.kind {
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
                match eval(cdar, env)?.kind {
                    ExprKind::List(list) => {
                        // TODO: implement consuming `into_iter()`
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::list::list;

    #[test]
    fn test_quote() {
        let evaluator = Evaluator::new();
        let env = evaluator.root_env();
        // (quote (1 2)) => (1 2)
        let result = quote("", &list!(list!(1, 2)), &env);
        assert_eq!(result, Ok(list!(1, 2).into()));
    }

    #[test]
    fn test_quasiquote() {
        let evaluator = Evaluator::new();
        let env = evaluator.root_env();

        env.define("x", 2);

        // `(0 1 ,x 3) => (0 1 2 3)
        let result = quasiquote(
            "",
            &list!(list!(0, 1, list!(intern("unquote"), intern("x")), 3)),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3).into()));
    }

    #[test]
    fn test_quasiquote_unquote() {
        let evaluator = Evaluator::with_builtin(); // make `num-add` available
        let env = evaluator.root_env();

        // (quasiquote (0 (unquote (+ 1 2)) 4)) => (0 3 4)
        let result = quasiquote(
            "",
            &list!(list!(
                0,
                list!(intern("unquote"), list!(intern("num-add"), 1, 2)),
                4
            )),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 3, 4).into()));
    }

    #[test]
    fn test_quasiquote_unquote_splicing() {
        let evaluator = Evaluator::new();
        let env = evaluator.root_env();

        // (quasiquote (0 (unquote-splicing (quote (1 2 3))) 4)) => (0 1 2 3 4)
        let result = quasiquote(
            "",
            &list!(list!(
                0,
                list!(
                    intern("unquote-splicing"),
                    list!(intern("quote"), list!(1, 2, 3))
                ),
                4
            )),
            &env,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3, 4).into()));
    }
}
