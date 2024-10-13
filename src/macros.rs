/// A macro for creating `crate::list::List`.
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
/// list!(intern("sym"), Expr::from("str")); // => (str "str")
/// ```
#[macro_export]
macro_rules! list {
    () => {
        $crate::list::List::Nil
    };

    ($car:literal $(, $cdr:expr)*) => {
        $crate::list::cons($crate::expr::Expr::from($car), list!($($cdr),*))
    };

    ($car:ident $(, $cdr:expr)*) => {
        $crate::list::cons($crate::expr::intern(stringify!($car)), list!($($cdr),*))
    };

    ($car:expr $(, $cdr:expr)*) => {
        $crate::list::cons($car, list!($($cdr),*))
    };
}

pub(crate) use list;

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

    #[test]
    fn test_list_sym() {
        format_eq!(list!(sym), "(sym)");
        format_eq!(list!(sym, "str", 1), "(sym \"str\" 1)");

        // For now, symbol can only be place as the first element
        // format_eq!(list!(sym, sym), "(sym sym)");
    }
}
