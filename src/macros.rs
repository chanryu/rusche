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
        expr::num,
        list::{cons, List},
    };

    use super::*;

    #[test]
    fn test_list() {
        assert_eq!(cons(num(0), List::Nil), list!(num(0)));
        assert_eq!(cons(num(0), cons(num(1), List::Nil)), list!(num(0), num(1)));
    }
}
