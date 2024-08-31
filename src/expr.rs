use crate::list::{cons, List};
use crate::proc::{NativeFunc, Proc};
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

    pub fn new_native_proc(func: NativeFunc) -> Self {
        Expr::Proc(Proc::Native(func))
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(value) => write!(f, "{}", value),
            Expr::Str(text) => write!(f, "\"{}\"", text), // TODO: escape as control chars
            Expr::Sym(name) => write!(f, "{}", name),
            Expr::Proc(func) => write!(f, "<#proc: {:?}>", func),
            Expr::List(list) => write!(f, "{}", list),
        }
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

#[cfg(test)]
pub mod shortcuts {
    use super::Expr;

    pub fn num<T: Into<f64>>(value: T) -> Expr {
        Expr::new_num(value.into())
    }

    pub fn str<T: Into<String>>(text: T) -> Expr {
        Expr::new_str(text.into())
    }

    pub fn sym<T: Into<String>>(text: T) -> Expr {
        Expr::new_sym(text)
    }
}

#[cfg(test)]
mod tests {
    use super::shortcuts::{num, str, sym};
    use super::*;
    use crate::{list::cons, macros::list};

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
        let list = list!(num(0));
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = list!(num(0), num(1), num(2));
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = list!(num(0), str("string"), sym("symbol"));
        assert_eq!(format!("{}", list), r#"(0 "string" symbol)"#);
    }

    #[test]
    fn test_vec_into_expr() {
        let v: Vec<Expr> = vec![num(1), num(2), list!(num(3), num(4)).into()];
        let expr: Expr = v.into();
        assert_eq!(format!("{}", expr), "(1 2 (3 4))");
    }
}
