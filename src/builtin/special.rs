use std::rc::Rc;

use crate::{
    eval::{eval, EvalContext, EvalError, EvalResult},
    expr::{intern, Expr, NIL},
    list::{cons, List},
    proc::Proc,
    utils::{get_exact_2_args, make_formal_args},
};

pub fn error(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if args.is_nil() {
        return Err(EvalError::from(format!(
            "{proc_name}: needs at least one argument."
        )));
    }

    let mut parts = Vec::new();
    for arg in args.iter() {
        let value = eval(arg, context)?;
        match value {
            Expr::Str(text, _) => parts.push(text),
            other => parts.push(other.to_string()),
        }
    }

    Err(EvalError::from(parts.join(" ")))
}

/// Builds the call form for `(apply proc list)`: the procedure in head position followed by
/// the already-evaluated elements of `list`.
///
/// Closures and natives receive each value wrapped in `(quote ...)`, so they do not
/// re-evaluate the arguments. Macros receive the values as-is. The evaluator evaluates the
/// returned form in tail position, so `apply` does not nest a call frame.
pub(crate) fn apply_form(
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> Result<Expr, EvalError> {
    let (proc_expr, args_expr) = get_exact_2_args(proc_name, args)?;

    let Expr::Proc(proc, _) = eval(proc_expr, context)? else {
        return Err(EvalError {
            message: format!("{proc_name}: `{proc_expr}` does not evaluate to a procedure."),
            span: proc_expr.span(),
        });
    };

    let Expr::List(arg_list, _) = eval(args_expr, context)? else {
        return Err(EvalError {
            message: format!("{proc_name}: `{args_expr}` does not evaluate to a list."),
            span: args_expr.span(),
        });
    };

    let call_args = match &proc {
        Proc::Macro { .. } => arg_list,
        _ => arg_list.iter().map(quote_expr).collect::<Vec<_>>().into(),
    };

    // A procedure value in head position evaluates to itself, so this is a regular call.
    Ok(cons(Expr::Proc(proc, None), call_args).into())
}

fn quote_expr(value: &Expr) -> Expr {
    cons(intern("quote"), cons(value.clone(), List::Nil)).into()
}

pub fn define(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();
    match iter.next() {
        Some(Expr::Sym(name, span)) => {
            let Some(expr) = iter.next() else {
                return Err(EvalError {
                    message: format!("{proc_name}: define expects a expression after symbol"),
                    span: *span,
                });
            };

            let value = match eval(expr, context)? {
                // `(define f (lambda ...))` -- give the anonymous closure a name so that
                // error messages can refer to it.
                Expr::Proc(
                    Proc::Closure {
                        name: None,
                        formal_args,
                        body,
                        outer_context,
                    },
                    span,
                ) => Expr::Proc(
                    Proc::Closure {
                        name: Some(name.clone()),
                        formal_args,
                        body,
                        outer_context,
                    },
                    span,
                ),
                value => value,
            };
            context.env.define(name, value);
            Ok(NIL)
        }
        Some(Expr::List(List::Cons(cons), _)) => {
            let Expr::Sym(name, _) = &cons.car else {
                return Err(EvalError {
                    message: format!("{proc_name}: expects a symbol for a procedure name"),
                    span: cons.car.span(),
                });
            };

            context.env.define(
                name,
                Expr::Proc(
                    Proc::Closure {
                        name: Some(name.to_string()),
                        formal_args: make_formal_args(&Expr::from(cons.cdr.clone()))?,
                        body: Rc::new(iter.into()),
                        outer_context: context.clone(),
                    },
                    args.span(),
                ),
            );
            Ok(NIL)
        }
        _ => Err(EvalError::from(format!(
            "{proc_name}: invalid form -- expected a symbol or a list."
        ))),
    }
}

pub fn defmacro(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();
    let expr = iter.next();
    let (macro_name, formal_args) = match expr {
        // (defmacro name (args) body)
        Some(Expr::Sym(macro_name, _)) => {
            let Some(expr) = iter.next() else {
                return Err(EvalError {
                    message: format!("{proc_name}: expected formal arguments after a macro name."),
                    span: args.span(),
                });
            };

            (macro_name, make_formal_args(expr)?)
        }
        // (defmacro (name args) body)
        Some(Expr::List(List::Cons(cons), _)) => {
            let Expr::Sym(macro_name, _) = &cons.car else {
                return Err(EvalError {
                    message: format!(
                        "{proc_name}: a macro name expected as the first element of the list."
                    ),
                    span: cons.car.span(),
                });
            };

            (macro_name, make_formal_args(&Expr::from(cons.cdr.clone()))?)
        }
        _ => {
            return Err(EvalError {
                message: format!("{proc_name}: invalid macro form -- expected a symbol or a list."),
                span: expr.map(|e| e.span()).unwrap_or(None),
            });
        }
    };

    context.env.define(
        macro_name,
        Expr::Proc(
            Proc::Macro {
                name: Some(macro_name.clone()),
                formal_args,
                body: Rc::new(iter.into()),
            },
            args.span(),
        ),
    );

    Ok(NIL)
}

pub fn eq(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (left, right) = get_exact_2_args(proc_name, args)?;

    Ok((eval(left, context)? == eval(right, context)?).into())
}

pub fn lambda(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();

    let Some(expr) = iter.next() else {
        return Err(EvalError {
            message: format!("{proc_name}: expected formal arguments."),
            span: args.span(),
        });
    };

    Ok(Expr::Proc(
        Proc::Closure {
            name: None,
            formal_args: make_formal_args(expr)?,
            body: Rc::new(iter.into()),
            outer_context: context.clone(),
        },
        args.span(),
    ))
}

pub fn set(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (name_expr, value_expr) = get_exact_2_args(proc_name, args)?;

    let Expr::Sym(name, _) = name_expr else {
        return Err(EvalError {
            message: format!("{proc_name}: expects a symbol as the first argument"),
            span: name_expr.span(),
        });
    };

    let value = eval(value_expr, context)?;
    if !context.env.update(name, value) {
        return Err(EvalError {
            message: format!("{proc_name}: `{name}` is not defined."),
            span: name_expr.span(),
        });
    }

    Ok(NIL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::*;

    #[test]
    fn test_define() {
        setup_native_proc_test!(define, env);

        // (define name "value")
        let ret = define(list!(intern("name"), "value"));
        assert_eq!(ret, Ok(NIL));
        assert_eq!(env.lookup("name"), Some("value".into()));

        // (define 1 "value") -> Err
        assert!(define(list!(1, "value")).is_err());

        // (define name) -> Err
        assert!(define(list!(intern("name"))).is_err());

        // (define (1 a b) '()) -> Err
        assert!(define(list!(list!(1, intern("a"), intern("b")), NIL)).is_err());

        // (define (name 1 b) '()) -> Err
        assert!(define(list!(list!(intern("name"), 1, intern("b")), NIL)).is_err());
    }

    #[test]
    fn test_defmacro() {
        setup_native_proc_test!(defmacro);

        // (defmacro x () ())
        assert!(defmacro(list!(intern("x"), list!(), list!())).is_ok());

        // (defmacro x) -> Err
        assert!(defmacro(list!(intern("x"))).is_err());

        // (defmacro add (a b) (+ a b))
        assert!(defmacro(list!(
            intern("add"),
            list!(intern("a"), intern("b")),
            list!(intern("+"), intern("a"), intern("b"))
        ))
        .is_ok());

        // (defmacro (add a b) (+ a b))
        assert!(defmacro(list!(
            list!(intern("add"), intern("a"), intern("b")),
            list!(intern("+"), intern("a"), intern("b"))
        ))
        .is_ok());

        // (defmacro) -> Err
        assert!(defmacro(list!()).is_err());

        // (defmacro x args ()) -> Ok, `args` receives every argument
        assert!(defmacro(list!(intern("x"), intern("args"), list!())).is_ok());

        // (defmacro x 1 ()) -> Err
        assert!(defmacro(list!(intern("x"), 1, list!())).is_err());

        // (defmacro (x 1) ()) -> Err
        assert!(defmacro(list!(intern("x"), list!(intern("a"), 1), list!())).is_err());

        // (defmacro add (a 1) (+ a 1)) -> Err
        assert!(defmacro(list!(
            intern("add"),
            list!(intern("a"), 1),
            list!(intern("+"), intern("a"), 1)
        ))
        .is_err());

        // (defmacro (add a 1) (+ a 1)) -> Err
        assert!(defmacro(list!(
            list!(intern("add"), intern("a"), 1),
            list!(intern("+"), intern("a"), 1)
        ))
        .is_err());
    }

    #[test]
    fn test_eq() {
        setup_native_proc_test!(eq);

        // (eq 1 1) => #t
        assert_ne!(eq(list!(1, 1)).unwrap(), NIL);
        // (eq 1 2) => ()
        assert_eq!(eq(list!(1, 2)).unwrap(), NIL);
        // (eq "str" "str") => #t
        assert_ne!(eq(list!("str", "str")).unwrap(), NIL);
        // (eq 1 "1") => ()
        assert_eq!(eq(list!(1, "1")).unwrap(), NIL);
    }

    #[test]
    fn test_set() {
        setup_native_proc_test!(set, env);

        env.define("name", "old-value");

        // (set! name "value")
        assert!(set(list!(intern("name"), "new-value")).is_ok());
        assert_eq!(env.lookup("name"), Some(Expr::from("new-value")));

        // (set! 1 "value") -> Err
        assert!(set(list!(1, "value")).is_err());
    }

    #[test]
    fn test_error() {
        setup_native_proc_test!(error);

        let err = error(list!("bad", 42)).unwrap_err();
        assert_eq!(err.message, "bad 42");

        let err = error(list!("only")).unwrap_err();
        assert_eq!(err.message, "only");

        assert!(error(list!()).is_err());
    }

    #[test]
    fn test_apply_form() {
        let evaluator = crate::eval::Evaluator::new();
        let context = evaluator.context();
        let apply =
            |args: List| apply_form("apply", &args, context).and_then(|f| eval(&f, context));

        context
            .env
            .define_native_proc("+", crate::builtin::num::add);
        context
            .env
            .define_native_proc("car", crate::builtin::list::car);

        assert_eq!(
            apply(list!(intern("+"), list!(intern("quote"), list!(1, 2, 3)))),
            Ok(num(6))
        );
        assert_eq!(
            apply(list!(
                intern("car"),
                list!(intern("quote"), list!(list!(1, 2, 3)))
            )),
            Ok(num(1))
        );
        assert!(apply(list!(1, list!(intern("quote"), list!()))).is_err());
        assert!(apply(list!(intern("+"), 1)).is_err());
        assert!(apply(list!(intern("+"))).is_err());
    }
}
