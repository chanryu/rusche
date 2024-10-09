use std::{
    any::Any,
    fmt::{self},
    rc::Rc,
};

use crate::{
    eval::EvalContext,
    list::{cons, List, ListIter},
    proc::Proc,
    span::Span,
};

pub type Foreign = Rc<dyn Any>;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(f64, Option<Span>),
    Str(String, Option<Span>),
    Sym(String, Option<Span>),
    Proc(Proc, Option<Span>),
    List(List, Option<Span>),

    Foreign(Foreign),

    /// A special case for tail-call optimization.
    TailCall {
        proc: Proc,
        args: List,
        context: EvalContext,
    },
}

pub const NIL: Expr = Expr::List(List::Nil, None);

impl Expr {
    pub fn is_atom(&self) -> bool {
        match self {
            Expr::List(List::Cons(_), _) => false,
            _ => true,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Expr::List(List::Nil, _) => true,
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_nil()
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Expr::Num(_, span)
            | Expr::Str(_, span)
            | Expr::Sym(_, span)
            | Expr::Proc(_, span)
            | Expr::List(_, span) => span.clone(),
            Expr::Foreign(_) => None,
            Expr::TailCall { .. } => None,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Num(lhs, _), Expr::Num(rhs, _)) => lhs == rhs,
            (Expr::Str(lhs, _), Expr::Str(rhs, _)) => lhs == rhs,
            (Expr::Sym(lhs, _), Expr::Sym(rhs, _)) => lhs == rhs,
            (Expr::Proc(lhs, _), Expr::Proc(rhs, _)) => lhs == rhs,
            (Expr::List(lhs, _), Expr::List(rhs, _)) => lhs == rhs,
            _ => false,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(value, _) => write!(f, "{}", value),
            Expr::Str(text, _) => write!(f, "\"{}\"", text), // TODO: escape control chars
            Expr::Sym(name, _) => write!(f, "{}", name),
            Expr::Proc(proc, _) => write!(f, "<{}>", proc.fingerprint()),
            Expr::List(list, _) => write!(f, "{}", list),
            Expr::Foreign(object) => write!(f, "<foreign: {:p}>", object),

            // TailCall is a special case and should not be displayed.
            Expr::TailCall { proc, .. } => panic!("Unexpected TailCall: {:?}", proc),
        }
    }
}

impl From<List> for Expr {
    fn from(val: List) -> Self {
        Expr::List(val, None)
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

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Expr::Str(value, None)
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Expr::from(value.to_string())
    }
}

impl From<i32> for Expr {
    fn from(value: i32) -> Self {
        Expr::Num(value as f64, None)
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr::Num(value, None)
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        if value {
            Expr::Num(1.0, None)
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
/// use rusche::expr::{intern, Expr};
///
/// let symbol = intern("foo");
/// assert_eq!(symbol, Expr::Sym(String::from("foo"), None));
/// ```
pub fn intern<T: Into<String>>(name: T) -> Expr {
    Expr::Sym(name.into(), None)
}

#[cfg(test)]
pub mod test_utils {
    use super::Expr;

    pub fn num<T: Into<f64>>(value: T) -> Expr {
        Expr::Num(value.into(), None)
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
