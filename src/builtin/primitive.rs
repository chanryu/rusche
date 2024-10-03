use crate::{
    eval::{eval, eval_tail, EvalContext, EvalError, EvalErrorCode, EvalResult},
    expr::{Expr, NIL},
    list::List,
    proc::Proc,
    utils::{get_2_or_3_args, get_exact_1_arg, get_exact_2_args, make_formal_args},
};

pub fn atom(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    Ok(eval(expr, context)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons), _) = eval(expr, context)? {
        Ok(cons.car.as_ref().clone())
    } else {
        Err(EvalError {
            code: EvalErrorCode::TypeMismatch,
            message: format!("{proc_name}: `{expr}` does not evaluate to a list."),
            span: expr.span(),
        })
    }
}

pub fn cdr(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons), _) = eval(expr, context)? {
        Ok(cons.cdr.as_ref().clone().into())
    } else {
        Err(EvalError {
            code: EvalErrorCode::TypeMismatch,
            message: format!("{proc_name}: `{expr}` does not evaluate to a list."),
            span: expr.span(),
        })
    }
}

pub fn cons(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (car, cdr) = get_exact_2_args(proc_name, args)?;

    let car = eval(car, context)?;
    let Expr::List(cdr, _) = eval(cdr, context)? else {
        return Err(EvalError {
            code: EvalErrorCode::TypeMismatch,
            message: format!("{proc_name}: `{cdr}` does not evaluate to a list."),
            span: cdr.span(),
        });
    };

    Ok(crate::list::cons(car, cdr).into())
}

pub fn define(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name, span)) => {
            let Some(expr) = iter.next() else {
                return Err(EvalError {
                    code: EvalErrorCode::TypeMismatch,
                    message: format!("{proc_name}: define expects a expression after symbol"),
                    span: *span,
                });
            };

            context.env.define(name, eval(expr, context)?);
            Ok(NIL)
        }
        Some(Expr::List(List::Cons(cons), _)) => {
            let Expr::Sym(name, _) = cons.car.as_ref() else {
                return Err(EvalError {
                    code: EvalErrorCode::TypeMismatch,
                    message: format!("{proc_name}: expects a symbol for a procedure name"),
                    span: cons.car.span(),
                });
            };

            context.env.define(
                name,
                Expr::Proc(
                    Proc::Closure {
                        name: Some(name.to_string()),
                        formal_args: make_formal_args(&cons.cdr)?,
                        body: Box::new(iter.into()),
                        outer_context: context.clone(),
                    },
                    None,
                ),
            );
            Ok(NIL)
        }
        _ => Err(EvalError {
            code: EvalErrorCode::InvalidForm,
            message: format!(
                "{proc_name}: invalid definition form -- expected a symbol or a list."
            ),
            span: None,
        }),
    }
}

pub fn defmacro(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();

    let (macro_name, formal_args) = match iter.next() {
        // (defmacro name (args) body)
        Some(Expr::Sym(macro_name, _)) => {
            let expr = iter.next();
            let Some(Expr::List(list, _)) = expr else {
                return Err(EvalError {
                    code: EvalErrorCode::TypeMismatch,
                    message: format!(
                        "{proc_name}: expected a list of formal arguments after a macro name."
                    ),
                    span: expr.map(|e| e.span()).unwrap_or(None),
                });
            };

            (macro_name, make_formal_args(list)?)
        }
        // (defmacro (name args) body)
        Some(Expr::List(List::Cons(cons), _)) => {
            let Expr::Sym(macro_name, _) = cons.car.as_ref() else {
                return Err(EvalError {
                    code: EvalErrorCode::TypeMismatch,
                    message: format!(
                        "{proc_name}: a macro name expected as the first element of the list."
                    ),
                    span: cons.car.span(),
                });
            };

            (macro_name, make_formal_args(&cons.cdr)?)
        }
        _ => {
            return Err(EvalError {
                code: EvalErrorCode::InvalidForm,
                message: format!("{proc_name}: invalid macro form -- expected a symbol or a list."),
                span: None,
            })
        }
    };

    context.env.define(
        macro_name,
        Expr::Proc(
            Proc::Macro {
                name: Some(macro_name.clone()),
                formal_args,
                body: Box::new(iter.into()),
            },
            None, // TODO: add span
        ),
    );

    Ok(NIL)
}

pub fn eq(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (left, right) = get_exact_2_args(proc_name, args)?;

    Ok((eval(left, context)? == eval(right, context)?).into())
}

pub fn eval_(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    eval_tail(&eval(expr, context)?, context)
}

pub fn if_(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (condition, then_clause, else_clause) = get_2_or_3_args(proc_name, args)?;

    if eval(condition, context)?.is_truthy() {
        eval_tail(then_clause, context)
    } else if let Some(else_clause) = else_clause {
        eval_tail(else_clause, context)
    } else {
        Ok(NIL)
    }
}

pub fn lambda(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();

    let expr = iter.next();
    let Some(Expr::List(list, _)) = expr else {
        return Err(EvalError {
            code: EvalErrorCode::InvalidForm,
            message: format!("{proc_name}: expected a list of formal arguments."),
            span: expr.map(|e| e.span()).unwrap_or(None),
        });
    };

    Ok(Expr::Proc(
        Proc::Closure {
            name: None,
            formal_args: make_formal_args(list)?,
            body: Box::new(iter.into()),
            outer_context: context.clone(),
        },
        None, // TODO: add span
    ))
}

pub fn set(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (name_expr, value_expr) = get_exact_2_args(proc_name, args)?;

    let Expr::Sym(name, _) = name_expr else {
        return Err(EvalError {
            code: EvalErrorCode::TypeMismatch,
            message: format!("{proc_name}: expects a symbol as the first argument"),
            span: name_expr.span(),
        });
    };

    context.env.update(name, eval(value_expr, context)?);

    Ok(NIL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;
    use crate::expr::intern;
    use crate::list::list;

    #[test]
    fn test_define() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (define name "value")
        let ret = define("", &list!(intern("name"), "value"), context);
        assert_eq!(ret, Ok(NIL));
        assert_eq!(context.env.lookup("name"), Some("value".into()));
    }

    #[test]
    fn test_eq() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (eq 1 1) => #t
        assert_ne!(eq("", &list!(1, 1), context).unwrap(), NIL);
        // (eq 1 2) => ()
        assert_eq!(eq("", &list!(1, 2), context).unwrap(), NIL);
        // (eq "str" "str") => #t
        assert_ne!(eq("", &list!("str", "str"), context).unwrap(), NIL);
        // (eq 1 "1") => ()
        assert_eq!(eq("", &list!(1, "1"), context).unwrap(), NIL);
    }
}
