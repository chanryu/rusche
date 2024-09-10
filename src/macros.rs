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
mod tests {
    use super::*;
    use crate::{
        expr::shortcuts::num,
        list::{cons, List},
    };

    #[test]
    fn test_list_macro() {
        // (cons 0 nil) => (list 0)
        assert_eq!(cons(num(0), List::Nil), list!(0));

        // (cons 0 (cons 1 nil)) => (list 0 1)
        assert_eq!(cons(num(0), cons(num(1), List::Nil)), list!(0, 1));

        // (cons 0 (cons (cons 1 nil) (cons 2 nil))) => (list 0 (list 1) 2)
        assert_eq!(
            cons(
                num(0),
                cons(cons(num(1), List::Nil), cons(num(2), List::Nil))
            ),
            list!(0, list!(1), 2)
        );
    }
}
