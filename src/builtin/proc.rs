use crate::{
    eval::{eval, EvalContext, EvalResult},
    expr::Expr,
    list::List,
    utils::get_exact_1_arg,
};

pub fn is_proc(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Proc(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
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
    fn test_is_proc() {
        setup_native_proc_test!(is_proc, env);

        env.define_native_proc("atom?", crate::builtin::list::is_atom);
        assert_eq!(is_proc(list!(intern("atom?"))), Ok(true.into()));
        assert_eq!(is_proc(list!(1)), Ok(false.into()));
        assert!(is_proc(list!()).is_err());
    }
}
