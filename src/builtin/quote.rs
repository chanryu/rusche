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
    let mut exprs = quasiquote_expr(expr, context)?;
    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Err(EvalError::from(format!(
            "{proc_name}: expects only 1 argument"
        )))
    }
}

fn quasiquote_expr(expr: &Expr, context: &EvalContext) -> Result<Vec<Expr>, EvalError> {
    let Expr::List(list, _) = expr else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name = match cons.car.as_ref() {
        Expr::Sym(name, _) => Some(name.as_str()),
        _ => None,
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
                        exprs.extend(list.iter().cloned());
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
                v.extend(quasiquote_expr(expr, context)?);
            }
            exprs.push(Expr::from(v));
        }
    }

    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::macros::*;

    #[test]
    fn test_quote() {
        setup_native_proc_test!(quote);

        // '(1 2) => (1 2)
        let result = quote(list!(list!(1, 2)));
        assert_eq!(result, Ok(list!(1, 2).into()));
    }

    #[test]
    fn test_quote_error() {
        setup_native_proc_test!(quote);

        // (quote 1 2) => error
        assert!(quote(list!(1, 2)).is_err());
    }

    #[test]
    fn test_quasiquote() {
        setup_native_proc_test!(quasiquote);

        // `(0 1 2) => (0 1 2)
        let result = quasiquote(list!(list!(0, 1, 2)));
        assert_eq!(result, Ok(list!(0, 1, 2).into()));
    }

    #[test]
    fn test_quasiquote_error() {
        setup_native_proc_test!(quasiquote);

        // `,@'(1 2) => error
        let result = quasiquote(list!(list!(
            intern(UNQUOTE_SPLICING),
            list!(intern(QUOTE), list!(1, 2))
        )));
        assert!(result.is_err());

        // (quasiquote 1 2) => error
        assert!(quasiquote(list!(1, 2)).is_err());
    }

    #[test]
    fn test_quasiquote_unquote() {
        setup_native_proc_test!(quasiquote, env);

        env.define_native_proc("+", crate::builtin::num::add);

        // `(0 ,(+ 1 2) 4) => (0 3 4)
        let result = quasiquote(list!(list!(
            0,
            list!(intern("unquote"), list!(intern("+"), 1, 2)),
            4
        )));
        assert_eq!(result, Ok(list!(0, 3, 4).into()));
    }

    #[test]
    fn test_quasiquote_unquote_error() {
        setup_native_proc_test!(quasiquote);

        // `(0 (unquote) 4) => error
        let result = quasiquote(list!(list!(0, list!(intern("unquote")), 4)));
        assert!(result.is_err());
    }

    #[test]
    fn test_quasiquote_unquote_splicing() {
        setup_native_proc_test!(quasiquote);

        // `(0 ,@'(1 2 3) 4) => (0 1 2 3 4)
        // (quasiquote (0 (unquote-splicing (quote (1 2 3))) 4)) => (0 1 2 3 4)
        let result = quasiquote(list!(list!(
            0,
            list!(
                intern(UNQUOTE_SPLICING),
                list!(intern(QUOTE), list!(1, 2, 3))
            ),
            4
        )));
        assert_eq!(result, Ok(list!(0, 1, 2, 3, 4).into()));
    }

    #[test]
    fn test_quasiquote_unquote_splicing_error() {
        setup_native_proc_test!(quasiquote);

        // `(0 ,@1 2) => error
        let result = quasiquote(list!(list!(
            0,
            list!(intern(UNQUOTE_SPLICING), list!(intern(QUOTE), 1)),
            2
        )));
        assert!(result.is_err());

        // `(0 (unquote-splicing) 2) => error
        let result = quasiquote(list!(list!(0, list!(intern(UNQUOTE_SPLICING)), 2)));
        assert!(result.is_err());
    }
}
