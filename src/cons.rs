use crate::{expr::Expr, list::List};

#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Box<Expr>,
    pub cdr: Box<List>,
}

impl Cons {
    pub fn new<T>(car: T, cdr: List) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            car: Box::new(car.into()),
            cdr: Box::new(cdr),
        }
    }

    pub fn cdar(&self) -> Option<&Expr> {
        if let List::Cons(cons) = self.cdr.as_ref() {
            Some(&cons.car)
        } else {
            None
        }
    }
}
