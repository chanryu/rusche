use crate::eval::{eval, ErrorKind, EvalContext, EvalError, EvalResult};
use crate::expr::{intern, Expr, NIL};
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
    let mut exprs = quasiquote_expr(expr, context, 1)?;
    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Err(EvalError::new(
            ErrorKind::Arity,
            format!("{proc_name}: expected 1 argument, got {}", exprs.len()),
        ))
    }
}

fn quasiquote_expr(
    expr: &Expr,
    context: &EvalContext,
    level: usize,
) -> Result<Vec<Expr>, EvalError> {
    let Expr::List(list, _) = expr else {
        return Ok(vec![expr.clone()]);
    };

    let List::Cons(cons) = list else {
        return Ok(vec![NIL]);
    };

    let car_name = match &cons.car {
        Expr::Sym(name, _) => Some(name.as_str()),
        _ => None,
    };

    let mut exprs = Vec::new();
    match car_name {
        Some(QUASIQUOTE) => {
            let Some(cadr) = cons.cadr() else {
                return Err(EvalError::new(
                    ErrorKind::Arity,
                    format!("{QUASIQUOTE}: missing argument"),
                )
                .with_span(expr.span()));
            };
            let processed = expect_one(quasiquote_expr(cadr, context, level + 1)?, QUASIQUOTE)?;
            exprs.push(Expr::from(vec![intern(QUASIQUOTE), processed]));
        }
        Some(UNQUOTE) => {
            let Some(cadr) = cons.cadr() else {
                return Err(EvalError::new(
                    ErrorKind::Arity,
                    format!("{UNQUOTE}: missing argument"),
                )
                .with_span(expr.span()));
            };
            if level == 1 {
                exprs.push(eval(cadr, context)?);
            } else {
                let processed = expect_one(quasiquote_expr(cadr, context, level - 1)?, UNQUOTE)?;
                exprs.push(Expr::from(vec![intern(UNQUOTE), processed]));
            }
        }
        Some(UNQUOTE_SPLICING) => {
            let Some(cadr) = cons.cadr() else {
                return Err(EvalError::new(
                    ErrorKind::Arity,
                    format!("{UNQUOTE_SPLICING}: missing argument"),
                )
                .with_span(expr.span()));
            };
            if level == 1 {
                match eval(cadr, context)? {
                    Expr::List(list, _) => {
                        // TODO: implement consuming `into_iter()`
                        exprs.extend(list.iter().cloned());
                    }
                    value => {
                        return Err(EvalError::new(
                            ErrorKind::Type,
                            format!(
                                "{UNQUOTE_SPLICING}: `{cadr}` evaluated to `{value}`, expected a list"
                            ),
                        )
                        .with_span(cadr.span()));
                    }
                }
            } else {
                let processed =
                    expect_one(quasiquote_expr(cadr, context, level - 1)?, UNQUOTE_SPLICING)?;
                exprs.push(Expr::from(vec![intern(UNQUOTE_SPLICING), processed]));
            }
        }
        _ => {
            let mut v = Vec::with_capacity(list.len());
            for expr in list.iter() {
                v.extend(quasiquote_expr(expr, context, level)?);
            }
            exprs.push(Expr::from(v));
        }
    }

    Ok(exprs)
}

fn expect_one(mut exprs: Vec<Expr>, form: &str) -> Result<Expr, EvalError> {
    if exprs.len() == 1 {
        Ok(exprs.remove(0))
    } else {
        Err(EvalError::new(
            ErrorKind::Arity,
            format!("{form}: expected 1 argument, got {}", exprs.len()),
        ))
    }
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

    #[test]
    fn test_quasiquote_nested() {
        setup_native_proc_test!(quasiquote, env);

        env.define_native_proc("+", crate::builtin::num::add);

        // ``(a ,,(+ 1 2)) => (quasiquote (a (unquote 3)))
        let result = quasiquote(list!(list!(
            intern(QUASIQUOTE),
            list!(
                intern("a"),
                list!(
                    intern(UNQUOTE),
                    list!(intern(UNQUOTE), list!(intern("+"), 1, 2))
                )
            )
        )));
        assert_eq!(
            result,
            Ok(list!(
                intern(QUASIQUOTE),
                list!(intern("a"), list!(intern(UNQUOTE), 3))
            )
            .into())
        );
    }

    #[test]
    fn test_nested_quasiquote_missing_arguments() {
        setup_native_proc_test!(quasiquote);

        // `` -- nested quasiquote with no argument: (quasiquote (quasiquote))
        let err = quasiquote(list!(list!(intern(QUASIQUOTE)))).unwrap_err();
        assert!(err.message.contains("quasiquote: missing argument"));

        // ``(,(unquote)) -- nested unquote with no argument
        let err = quasiquote(list!(list!(
            intern(QUASIQUOTE),
            list!(intern(UNQUOTE))
        )))
        .unwrap_err();
        assert!(err.message.contains("unquote: missing argument"));

        // ``(,(unquote-splicing)) -- nested splicing with no argument
        let err = quasiquote(list!(list!(
            intern(QUASIQUOTE),
            list!(intern(UNQUOTE_SPLICING))
        )))
        .unwrap_err();
        assert!(err.message.contains("unquote-splicing: missing argument"));
    }

    #[test]
    fn test_nested_unquote_splicing_preserves_form() {
        setup_native_proc_test!(quasiquote);

        // ``(,@'(1 2)) => (quasiquote (unquote-splicing (quote (1 2))))
        let result = quasiquote(list!(list!(
            intern(QUASIQUOTE),
            list!(
                intern(UNQUOTE_SPLICING),
                list!(intern(QUOTE), list!(1, 2))
            )
        )));
        assert_eq!(
            result,
            Ok(list!(
                intern(QUASIQUOTE),
                list!(
                    intern(UNQUOTE_SPLICING),
                    list!(intern(QUOTE), list!(1, 2))
                )
            )
            .into())
        );
    }

    #[test]
    fn test_nested_unquote_preserves_form() {
        setup_native_proc_test!(quasiquote);

        // ``(,x) => (quasiquote (unquote x))
        let result = quasiquote(list!(list!(
            intern(QUASIQUOTE),
            list!(intern(UNQUOTE), intern("x"))
        )));
        assert_eq!(
            result,
            Ok(list!(
                intern(QUASIQUOTE),
                list!(intern(UNQUOTE), intern("x"))
            )
            .into())
        );
    }
}
