use crate::{
    eval::{eval, EvalContext, EvalError, EvalResult},
    expr::Expr,
    list::List,
    utils::{get_exact_1_arg, get_exact_2_args},
};

pub fn is_atom(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    Ok(eval(expr, context)?.is_atom().into())
}

pub fn car(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;

    if let Expr::List(List::Cons(cons), _) = eval(expr, context)? {
        Ok(cons.car.clone())
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
        Ok(cons.cdr.clone().into())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::expr::NIL;
    use crate::macros::*;

    #[test]
    fn test_is_atom() {
        setup_native_proc_test!(is_atom);

        // (atom? 1) => #t
        assert_eq!(is_atom(list!(1)), Ok(true.into()));

        // (atom? "str") => #t
        assert_eq!(is_atom(list!("str")), Ok(true.into()));

        // (atom? '()) => #t
        assert_eq!(is_atom(list!(list!(intern("quote"), NIL))), Ok(true.into()));

        // (atom? '(1 2 3)) => #f
        assert_eq!(
            is_atom(list!(list!(intern("quote"), list!(1, 2, 3)))),
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

        // (cons 1 2) => err (cdr is not a list)
        assert!(cons(list!(1, 2)).is_err());

        // (cons 1 2 3) => err (wrong number of arguments)
        assert!(cons(list!(1, 2, 3)).is_err());
    }
}
