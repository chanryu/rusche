use crate::{
    eval::{eval, EvalContext, EvalResult},
    expr::Expr,
    list::List,
    utils::get_exact_1_arg,
};

pub fn is_sym(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Sym(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::macros::*;

    #[test]
    fn test_is_sym() {
        setup_native_proc_test!(is_sym);

        assert_eq!(
            is_sym(list!(list!(intern("quote"), intern("foo")))),
            Ok(true.into())
        );
        assert_eq!(is_sym(list!(1)), Ok(false.into()));
        assert_eq!(is_sym(list!("foo")), Ok(false.into()));
        assert!(is_sym(list!()).is_err());
    }
}
