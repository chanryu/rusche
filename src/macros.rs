#[macro_export]
macro_rules! list {
    // Base case: when no items are provided, return the empty list.
    () => {
        List::Nil
    };
    // Recursive case: when at least one item is provided, recursively build the list.
    ($first:expr $(, $rest:expr)*) => {
        cons($first, list!($($rest),*))
    };
}

pub(crate) use list;

#[cfg(test)]
mod tests {
    use crate::{
        expr::shortcuts::num,
        list::{cons, List},
    };

    use super::*;

    #[test]
    fn test_list() {
        // (cons 0 nil) => (list 0)
        assert_eq!(cons(num(0), List::Nil), list!(num(0)));

        // (cons 0 (cons 1 nil)) => (list 0 1)
        assert_eq!(cons(num(0), cons(num(1), List::Nil)), list!(num(0), num(1)));

        // (cons 0 (cons (cons 1 nil) (cons 2 nil))) => (list 0 (list 1) 2)
        assert_eq!(
            cons(
                num(0),
                cons(cons(num(1), List::Nil), cons(num(2), List::Nil))
            ),
            list!(num(0), list!(num(1)), num(2))
        );
    }
}
