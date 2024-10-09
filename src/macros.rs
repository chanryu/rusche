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
