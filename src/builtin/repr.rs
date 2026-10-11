use crate::{
    eval::{eval, EvalContext, EvalResult},
    expr::Expr,
    list::List,
    utils::get_exact_1_arg,
};

/// `(repr expr)` -- the source form of a value, as a string.
pub fn repr(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    let value = eval(expr, context)?;
    Ok(Expr::from(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::macros::*;

    #[test]
    fn test_repr() {
        setup_native_proc_test!(repr);

        assert_eq!(repr(list!("hi")), Ok(Expr::from(r#""hi""#)));
        assert_eq!(repr(list!("hi\n")), Ok(Expr::from("\"hi\\n\"")));
        assert_eq!(repr(list!(1)), Ok(Expr::from("1")));
        assert_eq!(
            repr(list!(list!(intern("quote"), intern("foo")))),
            Ok(Expr::from("foo"))
        );
        assert!(repr(list!()).is_err());
        assert!(repr(list!(1, 2)).is_err());
    }
}
