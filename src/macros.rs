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
    macro_rules! assert_list {
        ($list:expr, $result:literal) => {
            assert_eq!(format!("{}", $list), $result);
        };
    }

    #[test]
    fn test_list_nil() {
        assert_list!(list!(), "()");
    }

    #[test]
    fn test_list_num() {
        assert_list!(list!(1), "(1)");
        assert_list!(list!(1, 2), "(1 2)");
        assert_list!(list!(1, 2, list!(3, 4)), "(1 2 (3 4))");
        // assert_list!(list!("str", sym, 1), "(sym \"str\" 1)");
    }

    #[test]
    fn test_list_str() {
        assert_list!(list!("str"), "(\"str\")");
        assert_list!(list!("str", "str"), "(\"str\" \"str\")");
    }

    #[test]
    fn test_list_sym() {
        assert_list!(list!(sym), "(sym)");
        assert_list!(list!(sym, "str", 1), "(sym \"str\" 1)");

        // For now, symbol can only be place as the first element
        // assert_list!(list!(sym, sym), "(sym sym)");
    }
}
