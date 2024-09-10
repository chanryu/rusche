use crate::cons::Cons;
use crate::expr::Expr;
use std::fmt;
use std::iter::Iterator;

#[derive(Clone, Debug, PartialEq)]
pub enum List {
    Cons(Cons),
    Nil,
}

impl List {
    pub fn iter(&self) -> ListIter {
        ListIter::new(self)
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_nil(&self) -> bool {
        if let List::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn cdr(&self) -> Option<&List> {
        if let List::Cons(cons) = &self {
            Some(cons.cdr.as_ref())
        } else {
            None
        }
    }
}

impl<'a> From<ListIter<'a>> for List {
    fn from(val: ListIter<'a>) -> Self {
        val.list.clone()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_list(f, self, true)
    }
}

fn write_list(f: &mut fmt::Formatter<'_>, list: &List, is_top_level: bool) -> fmt::Result {
    if is_top_level {
        write!(f, "(")?;
    }
    if let List::Cons(cons) = list {
        if is_top_level {
            write!(f, "{}", cons.car)?;
        } else {
            write!(f, " {}", cons.car)?;
        }

        write_list(f, cons.cdr.as_ref(), false)?
    }
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}

pub struct ListIter<'a> {
    list: &'a List,
}

impl<'a> ListIter<'a> {
    pub fn new(list: &'a List) -> Self {
        Self { list }
    }
}

impl<'a> Iterator for ListIter<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        if let List::Cons(cons) = self.list {
            let car = &cons.car;
            self.list = &cons.cdr;
            Some(car)
        } else {
            None
        }
    }
}

pub fn cons<T>(car: T, cdr: List) -> List
where
    T: Into<Expr>,
{
    List::Cons(Cons::new(car, cdr))
}

#[macro_export]
macro_rules! list {
    // Base case: when no items are provided, return the empty list.
    () => {
        List::Nil
    };

    ($first:literal $(, $rest:expr)*) => {
        cons(Expr::new_num($first), list!($($rest),*))
    };

    // Recursive case: when at least one item is provided, recursively build the list.
    ($first:expr $(, $rest:expr)*) => {
        cons($first, list!($($rest),*))
    };
}

pub(crate) use list;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, str, sym};
    use crate::list::list;

    #[test]
    fn test_display() {
        let list = list!(1, 2, list!(3, sym("sym"), str("str")));
        assert_eq!(format!("{}", list), "(1 2 (3 sym \"str\"))");
    }

    #[test]
    fn test_iter() {
        let list = list!(1, 2, 3);
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&num(1)));
        assert_eq!(iter.next(), Some(&num(2)));
        assert_eq!(iter.next(), Some(&num(3)));
        assert_eq!(iter.next(), None);
    }

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
