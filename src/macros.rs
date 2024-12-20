/// A macro for creating [`crate::list::List`].
///
/// ```
/// use rusche::list;
/// use rusche::expr::{Expr, intern};
///
/// list!(); // => ()
/// list!(1); // => (1)
/// list!(1, 2); // => (1 2)
/// list!(1, 2, list!(3, 4)); // => (1 2 (3 4))
/// list!("str"); // => ("str")
/// list!("str", "str"); // => ("str" "str")
/// list!(intern("sym"), Expr::from("str")); // => (sym "str")
/// ```
#[macro_export]
macro_rules! list {
    () => {
        $crate::list::List::Nil
    };

    ($car:literal $(, $cdr:expr)*) => {
        $crate::list::cons($crate::expr::Expr::from($car), list!($($cdr),*))
    };

    ($car:expr $(, $cdr:expr)*) => {
        $crate::list::cons($car, list!($($cdr),*))
    };
}

pub(crate) use list;

#[cfg(test)]
macro_rules! setup_native_proc_test {
    ($fn_name:ident) => {
        let evaluator = $crate::eval::Evaluator::new();
        let context = evaluator.context();
        let $fn_name = |args| $fn_name(stringify!($fn_name), &args, context);
    };
    ($fn_name:ident, $env_name:ident) => {
        let evaluator = $crate::eval::Evaluator::new();
        let context = evaluator.context();
        let $fn_name = |args| $fn_name(stringify!($fn_name), &args, context);
        let $env_name = &context.env;
    };
}

#[cfg(test)]
pub(crate) use setup_native_proc_test;

#[cfg(test)]
macro_rules! tok {
    ($token_case:ident) => {{
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        Token::$token_case(crate::span::Loc::new(rng.gen(), rng.gen()))
    }};
    ($token_case:ident($value:expr)) => {
        Token::$token_case(
            $value.into(),
            crate::span::Span::new(crate::span::Loc::new(0, 0), crate::span::Loc::new(0, 1)),
        )
    };
}

#[cfg(test)]
pub(crate) use tok;

#[cfg(test)]
mod tests {
    macro_rules! format_eq {
        ($list:expr, $result:literal) => {
            assert_eq!(format!("{}", $list), $result);
        };
    }

    #[test]
    fn test_list_nil() {
        format_eq!(list!(), "()");
    }

    #[test]
    fn test_list_num() {
        format_eq!(list!(1), "(1)");
        format_eq!(list!(1, 2), "(1 2)");
        format_eq!(list!(1, 2, list!(3, 4)), "(1 2 (3 4))");
    }

    #[test]
    fn test_list_str() {
        format_eq!(list!("str"), "(\"str\")");
        format_eq!(list!("str", "str"), "(\"str\" \"str\")");
    }
}
