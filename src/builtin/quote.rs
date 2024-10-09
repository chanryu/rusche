use crate::eval::{eval, EvalContext, EvalError, EvalResult};
use crate::expr::{Expr, NIL};
use crate::list::List;
use crate::utils::get_exact_1_arg;

pub const QUOTE: &str = "quote";
pub const QUASIQUOTE: &str = "quasiquote";
pub const UNQUOTE: &str = "unquote";
pub const UNQUOTE_SPLICING: &str = "unquote-splicing";

pub fn quote(proc_name: &str, args: &List, _context: &EvalContext) -> EvalResult {
    Ok(get_exact_1_arg(proc_name, args)?.clone())
}

pub fn quasiquote(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    let mut exprs = quasiquote_expr(proc_name, expr, context)?;
    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Err(EvalError::from(format!(
            "{proc_name}: expects only 1 argument"
        )))
    }
}

fn quasiquote_expr(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Vec<Expr>, EvalError> {
    let Expr::List(list, _) = expr else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name = if let Expr::Sym(name, _) = cons.car.as_ref() {
        Some(name.as_str())
    } else {
        None
    };

    let mut exprs = Vec::new();
    match car_name {
        Some(UNQUOTE) => {
            if let Some(cdar) = cons.cdar() {
                exprs.push(eval(cdar, context)?);
            } else {
                return Err(EvalError {
                    message: format!("{UNQUOTE}: missing argument"),
                    span: expr.span(),
                });
            }
        }
        Some(UNQUOTE_SPLICING) => {
            if let Some(cdar) = cons.cdar() {
                match eval(cdar, context)? {
                    Expr::List(list, _) => {
                        // TODO: implement consuming `into_iter()`
                        exprs.extend(list.iter().map(|e| e.clone()));
                    }
                    _ => {
                        return Err(EvalError {
                            message: format!(
                                "{UNQUOTE_SPLICING}: `{cdar}` does not evaluate to a list"
                            ),
                            span: cdar.span(),
                        });
                    }
                }
            } else {
                return Err(EvalError {
                    message: format!("{UNQUOTE_SPLICING}: argument missing"),
                    span: expr.span(),
                });
            }
        }
        _ => {
            let mut v = Vec::with_capacity(list.len());
            for expr in list.iter() {
                v.extend(quasiquote_expr(proc_name, expr, context)?);
            }
            exprs.push(Expr::from(v));
        }
    }

    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::macros::list;

    #[test]
    fn test_quote() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();
        // (quote (1 2)) => (1 2)
        let result = quote(QUOTE, &list!(list!(1, 2)), context);
        assert_eq!(result, Ok(list!(1, 2).into()));
    }

    #[test]
    fn test_quasiquote() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        context.env.define("x", 2);

        // `(0 1 ,x 3) => (0 1 2 3)
        let result = quasiquote(
            QUASIQUOTE,
            &list!(list!(0, 1, list!(intern(UNQUOTE), intern("x")), 3)),
            context,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3).into()));
    }

    #[test]
    fn test_quasiquote_unquote() {
        let evaluator = Evaluator::with_builtin(); // make `num-add` available
        let context = evaluator.context();

        // (quasiquote (0 (unquote (+ 1 2)) 4)) => (0 3 4)
        let result = quasiquote(
            QUASIQUOTE,
            &list!(list!(
                0,
                list!(intern(UNQUOTE), list!(intern("num-add"), 1, 2)),
                4
            )),
            context,
        );
        assert_eq!(result, Ok(list!(0, 3, 4).into()));
    }

    #[test]
    fn test_quasiquote_unquote_splicing() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (quasiquote (0 (unquote-splicing (quote (1 2 3))) 4)) => (0 1 2 3 4)
        let result = quasiquote(
            QUASIQUOTE,
            &list!(list!(
                0,
                list!(
                    intern(UNQUOTE_SPLICING),
                    list!(intern(QUOTE), list!(1, 2, 3))
                ),
                4
            )),
            context,
        );
        assert_eq!(result, Ok(list!(0, 1, 2, 3, 4).into()));
    }
}
