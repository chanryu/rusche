use crate::{
    eval::{eval, eval_tail, EvalContext, EvalError, EvalResult},
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
            let expr = iter.next();
            let Some(Expr::List(list, _)) = expr else {
                return Err(EvalError {
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
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::*;

    #[test]
    fn test_atom() {
        setup_native_proc_test!(atom);

        // (atom 1) => #t
        assert_eq!(atom(list!(1)), Ok(true.into()));

        // (atom "str") => #t
        assert_eq!(atom(list!("str")), Ok(true.into()));

        // (atom '()) => #t
        assert_eq!(atom(list!(list!(intern("quote"), NIL))), Ok(true.into()));

        // (atom '(1 2 3)) => #f
        assert_eq!(
            atom(list!(list!(intern("quote"), list!(1, 2, 3)))),
            Ok(false.into())
        );
    }

    #[test]
    fn test_car() {
        setup_native_proc_test!(car);

        // (car '(1 2 3)) => 1
        assert_eq!(
            car(list!(list!(intern("quote"), list!(1, 2, 3)))),
            Ok(num(1))
        );

        // (car (1 2 3)) => err
        assert!(car(list!(list!(1, 2, 3))).is_err());

        // (car 1) => err
        assert!(car(list!(1)).is_err());

        // (car 1 2) => err
        assert!(car(list!(1, 2)).is_err());
    }

    #[test]
    fn test_cdr() {
        setup_native_proc_test!(cdr);

        // (cdr '(1 2 3)) => (2 3)
        assert_eq!(
            cdr(list!(list!(intern("quote"), list!(1, 2, 3)))),
            Ok(list!(2, 3).into())
        );

        // (cdr (1 2 3)) => err
        assert!(cdr(list!(list!(1, 2, 3))).is_err());

        // (cdr 1) => err
        assert!(cdr(list!(1)).is_err());

        // (cdr '(1 2 3) 4) => err
        assert!(cdr(list!(list!(intern("quote"), list!(1, 2, 3)), 4)).is_err());
    }

    #[test]
    fn test_cons() {
        setup_native_proc_test!(cons);

        // (cons 1 '(2 3)) => (1 2 3)
        assert_eq!(
            cons(list!(1, list!(intern("quote"), list!(2, 3)))),
            Ok(list!(1, 2, 3).into())
        );

        // (car 1 2) => err (cdr is not a list)
        assert!(cons(list!(1, 2)).is_err());

        // (car 1 2 3) => err (wrong number of arguments)
        assert!(cons(list!(1, 2, 3)).is_err());
    }

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

        // (defmacro x a ()) -> Err
        assert!(defmacro(list!(intern("x"), intern("a"), list!())).is_err());

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
}
