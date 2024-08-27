use crate::expr::Expr;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Box<Expr>,
    pub cdr: Box<List>,
}

impl Cons {
    pub fn new(car: Expr, cdr: List) -> Self {
        Self {
            car: Box::new(car),
            cdr: Box::new(cdr),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum List {
    Cons(Cons),
    Nil,
}

impl List {
    pub fn new_cons(car: Expr, cdr: List) -> Self {
        List::Cons(Cons::new(car, cdr))
    }

    pub fn iter(&self) -> ListIter {
        ListIter::new(self)
    }

    pub fn collect(&self) -> Vec<&Expr> {
        self.iter().collect::<Vec<_>>()
    }

    pub fn car(&self) -> Option<&Expr> {
        if let List::Cons(cons) = &self {
            Some(cons.car.as_ref())
        } else {
            None
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

impl Into<Expr> for List {
    fn into(self) -> Expr {
        Expr::List(self)
    }
}

pub fn cons<T>(car: T, cdr: List) -> List
where
    T: Into<Expr>,
{
    List::new_cons(car.into(), cdr)
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
