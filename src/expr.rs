use crate::list::{cons, List, ListIter};
use crate::proc::{NativeFunc, Proc};
use crate::token::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Proc),
    List(List),
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ExprKind::Num(value) => write!(f, "{}", value),
            ExprKind::Str(text) => write!(f, "\"{}\"", text), // TODO: escape control chars
            ExprKind::Sym(name) => write!(f, "{}", name),
            ExprKind::Proc(proc) => write!(f, "<{}>", proc.fingerprint()),
            ExprKind::List(list) => write!(f, "{}", list),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Option<Span>,
}

pub const NIL: Expr = Expr {
    kind: ExprKind::List(List::Nil),
    span: None,
};

impl Expr {
    pub fn new(kind: ExprKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }

    pub fn new_num<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        Self {
            kind: ExprKind::Num(value.into()),
            span: None,
        }
    }

    pub fn new_str<T: Into<String>>(text: T) -> Self {
        Self {
            kind: ExprKind::Str(text.into()),
            span: None,
        }
    }

    pub fn new_sym<T: Into<String>>(name: T) -> Self {
        Self {
            kind: ExprKind::Sym(name.into()),
            span: None,
        }
    }

    pub fn new_native_proc(name: &str, func: NativeFunc) -> Self {
        Self {
            kind: ExprKind::Proc(Proc::Native {
                name: name.to_owned(),
                func: func,
            }),
            span: None,
        }
    }

    pub fn is_atom(&self) -> bool {
        match self.kind {
            ExprKind::List(List::Cons(_)) => false,
            _ => true,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self.kind {
            ExprKind::List(List::Nil) => true,
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_nil()
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl From<List> for Expr {
    fn from(val: List) -> Self {
        Expr {
            kind: ExprKind::List(val),
            span: None,
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

impl<'a> From<ListIter<'a>> for Expr {
    fn from(value: ListIter) -> Self {
        value.map(|expr| expr.clone()).collect::<Vec<_>>().into()
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Expr {
            kind: ExprKind::Str(value.to_string()),
            span: None,
        }
    }
}

impl From<i32> for Expr {
    fn from(value: i32) -> Self {
        Expr {
            kind: ExprKind::Num(value as f64),
            span: None,
        }
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr {
            kind: ExprKind::Num(value),
            span: None,
        }
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        if value {
            Expr {
                kind: ExprKind::Num(1.0),
                span: None,
            }
        } else {
            NIL
        }
    }
}

/// Interns a string into an `Expr::Sym`.
///
/// This function takes a string and converts it into an `Expr::Sym`. The string is
/// converted into an owned `String` and then wrapped in an `Expr::Sym` variant.
///
/// # Examples
///
/// ```
/// use rusp::expr::{intern, Expr};
///
/// let symbol = intern("foo");
/// assert_eq!(symbol, Expr::new_sym("foo"));
/// ```
pub fn intern<T: Into<String>>(name: T) -> Expr {
    Expr {
        kind: ExprKind::Sym(name.into()),
        span: None,
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn num<T: Into<f64>>(value: T) -> Expr {
        Expr::new_num(value)
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::num;
    use super::*;
    use crate::list::list;

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
        assert_eq!(format!("{}", Expr::from("str")), "\"str\"");
    }

    #[test]
    fn test_display_sym() {
        assert_eq!(format!("{}", intern("sym")), "sym");
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
        let list = list!(0, "str", intern("sym"));
        assert_eq!(format!("{}", list), r#"(0 "str" sym)"#);
    }

    #[test]
    fn test_expr_from_list() {
        assert_eq!(
            format!("{}", Expr::from(list!(list!(1, 2), 3, 4, intern("abc")))),
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
