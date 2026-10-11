use crate::{
    eval::{eval, ErrorKind, EvalContext, EvalError, EvalResult},
    expr::Expr,
    list::List,
    utils::{eval_into_num, eval_into_str, get_exact_1_arg},
};

pub fn num_to_str(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let value = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::Str(Expr::Num(value, None).to_string(), None))
}

pub fn str_to_num(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;

    match text.parse::<f64>() {
        Ok(num) => Ok(Expr::Num(num, None)),
        Err(_) => Ok(false.into()),
    }
}

pub fn sym_to_str(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    match eval(expr, context)? {
        Expr::Sym(name, _) => Ok(Expr::Str(name.to_string(), None)),
        value => Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{expr}` evaluated to `{value}`, expected a symbol"),
        )
        .with_span(expr.span())),
    }
}

pub fn str_to_sym(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::Sym(crate::symbol::Symbol::intern(text), None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::*;

    #[test]
    fn test_num_to_str() {
        setup_native_proc_test!(num_to_str);

        assert_eq!(num_to_str(list!(123)), Ok(Expr::from("123")));
        assert_eq!(num_to_str(list!(1.5)), Ok(Expr::from("1.5")));
        assert!(num_to_str(list!("123")).is_err());
        assert!(num_to_str(list!()).is_err());
    }

    #[test]
    fn test_str_to_num() {
        setup_native_proc_test!(str_to_num);

        // (str->num "123") => 123
        assert_eq!(str_to_num(list!("123")), Ok(num(123)));

        // (str->num "abc") => false
        assert_eq!(str_to_num(list!("abc")), Ok(false.into()));

        // (str->num "123" "456") => error
        assert!(str_to_num(list!("123", "456")).is_err());

        // (str->num) => error
        assert!(str_to_num(list!()).is_err());

        // (str->num 123) => error
        assert!(str_to_num(list!(123)).is_err());

        // (str->num 'sym) => error
        assert!(str_to_num(list!(intern("sym"))).is_err());
    }

    #[test]
    fn test_sym_to_str() {
        setup_native_proc_test!(sym_to_str);

        assert_eq!(
            sym_to_str(list!(list!(intern("quote"), intern("foo")))),
            Ok(Expr::from("foo"))
        );
        assert!(sym_to_str(list!("foo")).is_err());
        assert!(sym_to_str(list!(1)).is_err());
    }

    #[test]
    fn test_str_to_sym() {
        setup_native_proc_test!(str_to_sym);

        assert_eq!(str_to_sym(list!("foo")), Ok(intern("foo")));
        assert_eq!(str_to_sym(list!("")), Ok(intern("")));
        assert!(str_to_sym(list!(1)).is_err());
    }
}
