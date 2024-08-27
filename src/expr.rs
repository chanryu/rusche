use crate::eval::{Env, EvalResult};
use crate::list::List;
use std::fmt;

pub type Func = fn(args: &List, env: &Env) -> EvalResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Func),
    List(List),
}

impl Expr {
    pub fn is_atom(&self) -> bool {
        match self {
            Expr::List(List::Cons(_)) => false,
            _ => true,
        }
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

pub fn num<T>(value: T) -> Expr
where
    T: Into<f64>,
{
    Expr::Num(value.into())
}

pub fn str<T: Into<String>>(text: T) -> Expr {
    Expr::Str(text.into())
}

pub fn sym<T: Into<String>>(text: T) -> Expr {
    Expr::Sym(text.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::cons;

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", Expr::List(List::Nil)), "()");
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
        let list = cons(num(0), List::Nil);
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = cons(num(0), cons(num(1), cons(num(2), List::Nil)));
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = cons(num(0), cons(str("str"), cons(sym("sym"), List::Nil)));
        assert_eq!(format!("{}", list), "(0 \"str\" sym)");
    }
}
