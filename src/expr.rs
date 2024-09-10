use crate::list::{cons, List, ListIter};
use crate::proc::Proc;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Proc),
    List(List),
}

pub const NIL: Expr = Expr::List(List::Nil);

impl Expr {
    pub fn is_atom(&self) -> bool {
        match self {
            Expr::List(List::Cons(_)) => false,
            _ => true,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Expr::List(List::Nil) => true,
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_nil()
    }

    pub fn new_num<T>(value: T) -> Expr
    where
        T: Into<f64>,
    {
        Expr::Num(value.into())
    }

    pub fn new_str<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self::Str(text.into())
    }

    pub fn new_sym<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self::Sym(text.into())
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(value) => write!(f, "{}", value),
            Expr::Str(text) => write!(f, "\"{}\"", text), // TODO: escape as control chars
            Expr::Sym(name) => write!(f, "{}", name),
            Expr::Proc(proc) => write!(f, "<{}>", proc.fingerprint()),
            Expr::List(list) => write!(f, "{}", list),
        }
    }
}

impl From<List> for Expr {
    fn from(val: List) -> Self {
        Expr::List(val)
    }
}

impl From<Vec<Expr>> for Expr {
    fn from(mut value: Vec<Expr>) -> Self {
        let mut list = List::Nil;
        while let Some(expr) = value.pop() {
            list = cons(expr, list);
        }
        list.into()
    }
}

impl<'a> From<ListIter<'a>> for Expr {
    fn from(value: ListIter) -> Self {
        value.map(|expr| expr.clone()).collect::<Vec<_>>().into()
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        if value {
            Expr::Num(1_f64)
        } else {
            NIL
        }
    }
}

#[cfg(test)]
pub mod shortcuts {
    use super::Expr;

    pub fn num<T: Into<f64>>(value: T) -> Expr {
        Expr::new_num(value)
    }

    pub fn str<T: Into<String>>(text: T) -> Expr {
        Expr::new_str(text)
    }

    pub fn sym<T: Into<String>>(text: T) -> Expr {
        Expr::new_sym(text)
    }
}

#[cfg(test)]
mod tests {
    use super::shortcuts::{num, str, sym};
    use super::*;
    use crate::macros::list;

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", NIL), "()");
    }

    #[test]
    fn test_display_num() {
        assert_eq!(format!("{}", num(0)), "0");
        assert_eq!(format!("{}", num(1)), "1");
        assert_eq!(format!("{}", num(1.2)), "1.2");
        assert_eq!(format!("{}", num(2.0)), "2");
    }

    #[test]
    fn test_display_str() {
        assert_eq!(format!("{}", str("str")), "\"str\"");
    }

    #[test]
    fn test_display_sym() {
        assert_eq!(format!("{}", sym("sym")), "sym");
    }

    #[test]
    fn test_display_list_1() {
        let list = list!(0);
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = list!(0, 1, 2);
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = list!(0, str("str"), sym("sym"));
        assert_eq!(format!("{}", list), r#"(0 "str" sym)"#);
    }

    #[test]
    fn test_expr_from_list() {
        assert_eq!(
            format!("{}", Expr::from(list!(list!(1, 2), 3, 4, sym("abc")))),
            "((1 2) 3 4 abc)"
        );
    }

    #[test]
    fn test_expr_from_vec() {
        let v: Vec<Expr> = vec![num(1), num(2), list!(3, 4).into()];
        assert_eq!(format!("{}", Expr::from(v)), "(1 2 (3 4))");
    }

    #[test]
    fn test_expr_from_bool() {
        assert_eq!(Expr::from(true), num(1));
        assert_eq!(Expr::from(false), NIL);
    }
}
