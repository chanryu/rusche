use crate::expr::Expr;
use crate::span::Span;
use std::fmt;
use std::iter::Iterator;
use std::rc::Rc;

/// The struct that represents a [cons cell](https://en.wikipedia.org/wiki/Cons) that contains a value and a reference to the next cons cell.
#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Expr,
    pub cdr: List,
}

impl Cons {
    pub fn new<T>(car: T, cdr: List) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            car: car.into(),
            cdr,
        }
    }

    /// Returns the second element, i.e. the `car` of the `cdr`.
    pub fn cadr(&self) -> Option<&Expr> {
        if let List::Cons(cons) = &self.cdr {
            Some(&cons.car)
        } else {
            None
        }
    }
}

/// The enum that represents a list which is either a cons cell or the empty list.
///
/// Cons cells are reference counted and immutable, so cloning a list -- which happens on
/// every variable lookup -- is O(1) and shares structure with the original.
#[derive(Clone, Debug, PartialEq)]
pub enum List {
    Cons(Rc<Cons>),
    Nil,
}

impl List {
    pub fn iter(&self) -> ListIter<'_> {
        ListIter::new(self)
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, List::Nil)
    }

    pub fn is_nil(&self) -> bool {
        self.is_empty()
    }

    /// Returns the span from the first element to the last one.
    ///
    /// Returns `None` if either end has no span, or if the spans are not in source order --
    /// which happens when a macro expansion mixes expressions from the macro definition with
    /// expressions from the call site.
    pub fn span(&self) -> Option<Span> {
        let mut iter = self.iter();

        let first_span = iter.next()?.span()?;
        let Some(last) = iter.last() else {
            return Some(first_span);
        };
        let last_span = last.span()?;

        if first_span.begin < last_span.end {
            Some(Span::new(first_span.begin, last_span.end))
        } else {
            None
        }
    }

    /// Returns a copy of the list with every span removed, recursively.
    pub(crate) fn without_spans(&self) -> List {
        self.iter()
            .map(Expr::without_spans)
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a> From<ListIter<'a>> for List {
    fn from(val: ListIter<'a>) -> Self {
        val.list.clone()
    }
}

impl From<Vec<Expr>> for List {
    fn from(mut value: Vec<Expr>) -> Self {
        let mut list = List::Nil;
        while let Some(expr) = value.pop() {
            list = cons(expr, list);
        }
        list
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

        write_list(f, &cons.cdr, false)?
    }
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}

/// An iterator that iterates over the elements of [`List`].
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

/// Create a new cons cell with the given value and the next cons cell.
pub fn cons<T, U>(car: T, cdr: U) -> List
where
    T: Into<Expr>,
    U: Into<List>,
{
    List::Cons(Rc::new(Cons::new(car, cdr.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::intern;
    use crate::expr::test_utils::num;
    use crate::macros::list;
    use crate::span::Loc;

    #[test]
    fn test_cons_cadr() {
        // (1 nil).cadr => None
        assert_eq!(Cons::new(Expr::from(1), List::Nil).cadr(), None);

        // (1 '(1 2)).cadr => Some(1)
        assert_eq!(
            Cons::new(Expr::from(1), list!(1, 2)).cadr(),
            Some(&Expr::from(1))
        );
    }

    #[test]
    fn test_list_from_vec_and_without_spans() {
        let span = Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)));
        let list = List::from(vec![
            Expr::Num(1.0, span),
            Expr::List(list!(Expr::Sym("a".into(), span)), span),
        ]);
        assert_eq!(format!("{}", list), "(1 (a))");

        let stripped = list.without_spans();
        assert_eq!(stripped, list); // PartialEq ignores spans

        fn has_no_spans(expr: &Expr) -> bool {
            expr.span().is_none()
                && match expr {
                    Expr::List(list, _) => list.iter().all(has_no_spans),
                    _ => true,
                }
        }
        assert!(!list.iter().all(has_no_spans));
        assert!(stripped.iter().all(has_no_spans));
    }

    #[test]
    fn test_list_is_nil() {
        assert!(List::Nil.is_nil());
        assert!(list!().is_nil());
        assert!(!list!(1).is_nil());
        assert!(!list!(1, 2).is_nil());
    }

    #[test]
    fn test_display() {
        let list = list!(1, 2, list!(3, "str", intern("sym")));
        assert_eq!(format!("{}", list), "(1 2 (3 \"str\" sym))");
    }

    #[test]
    fn test_list_span() {
        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), Some(Span::new(Loc::new(1, 1), Loc::new(1, 6))));

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, None),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), None);

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 3), Loc::new(1, 4)))),
            Expr::Num(3.0, None)
        );
        assert_eq!(args.span(), None);

        // (1 2 3)
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(1, 1), Loc::new(1, 2)))),
            Expr::Num(2.0, None),
            Expr::Num(3.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), Some(Span::new(Loc::new(1, 1), Loc::new(1, 6))));

        // Regression: elements out of source order (macro expansion mixing the call site
        // with the definition site) must not produce an inverted span.
        let args = list!(
            Expr::Num(1.0, Some(Span::new(Loc::new(5, 1), Loc::new(5, 2)))),
            Expr::Num(2.0, Some(Span::new(Loc::new(1, 5), Loc::new(1, 6))))
        );
        assert_eq!(args.span(), None);
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
        assert_eq!(cons(0, List::Nil), list!(0));

        // (cons 0 (cons 1 nil)) => (list 0 1)
        assert_eq!(cons(0, cons(1, List::Nil)), list!(0, 1));

        // (cons 0 (cons (cons 1 nil) (cons 2 nil))) => (list 0 (list 1) 2)
        assert_eq!(
            cons(0, cons(cons(1, List::Nil), cons(2, List::Nil))),
            list!(0, list!(1), 2)
        );
    }
}
