use crate::eval::{Env, EvalResult};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Box<Expr>,
    pub cdr: Box<Expr>,
}

impl Cons {
    pub fn new(car: Expr, cdr: Expr) -> Self {
        Self {
            car: Box::new(car),
            cdr: Box::new(cdr),
        }
    }
}

pub type Func = fn(expr: &Expr, env: &Env) -> EvalResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Func),
    List(Option<Cons>),
}

pub const NIL: Expr = Expr::List(None);

impl Expr {
    pub fn splat(&self) -> Vec<&Expr> {
        let mut argv = Vec::new();
        let mut args = self;
        loop {
            match args {
                Expr::List(None) => break,
                Expr::List(Some(cons)) => {
                    argv.push(cons.car.as_ref());
                    args = &cons.cdr;
                }
                _ => {
                    argv.push(args);
                    break;
                }
            }
        }
        argv
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(value) => write!(f, "{}", value),
            Expr::Str(text) => write!(f, "\"{}\"", text), // TODO: escape as control chars
            Expr::Sym(name) => write!(f, "{}", name),
            Expr::Proc(func) => write!(f, "<#proc: {:?}>", func),
            Expr::List(None) => write!(f, "()"),
            Expr::List(Some(cons)) => write_option_cons(f, cons, true),
        }
    }
}

fn write_option_cons(f: &mut fmt::Formatter<'_>, cons: &Cons, is_top_level: bool) -> fmt::Result {
    if is_top_level {
        write!(f, "(")?;
    }
    match cons.cdr.as_ref() {
        Expr::List(None) => write!(f, "{}", cons.car)?,
        Expr::List(Some(inner_cons)) => {
            write!(f, "{} ", cons.car)?;
            write_option_cons(f, inner_cons, false)?
        }
        _ => write!(f, "{} . {}", cons.car, cons.cdr)?,
    }
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}

pub struct ExprIter<'a> {
    current: &'a Expr,
}

impl<'a> ExprIter<'a> {
    pub fn new(args: &'a Expr) -> Self {
        Self { current: args }
    }
}

impl<'a> Iterator for ExprIter<'a> {
    type Item = &'a Expr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Expr::List(None) => None,
            Expr::List(Some(cons)) => {
                let car = &cons.car;
                self.current = &cons.cdr;
                Some(car)
            }
            _ => {
                // improper list or not a list
                let arg = self.current;
                self.current = &NIL;
                Some(arg)
            }
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn num<T>(value: T) -> Expr
    where
        T: Into<f64>,
    {
        Expr::Num(value.into())
    }

    pub fn str(text: &str) -> Expr {
        Expr::Str(String::from(text))
    }

    pub fn sym(name: &str) -> Expr {
        Expr::Sym(String::from(name))
    }

    pub fn cons(car: Expr, cdr: Expr) -> Expr {
        Expr::List(Some(Cons::new(car, cdr)))
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", Expr::List(None)), "()");
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
        let list = cons(num(0), NIL);
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = cons(num(0), cons(num(1), cons(num(2), NIL)));
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = cons(num(0), cons(str("str"), cons(sym("sym"), NIL)));
        assert_eq!(format!("{}", list), "(0 \"str\" sym)");
    }

    #[test]
    fn test_display_improper_list_1() {
        let list = cons(num(0), num(1));
        assert_eq!(format!("{}", list), "(0 . 1)");
    }

    #[test]
    fn test_display_improper_list_2() {
        let list = cons(num(0), cons(num(1), num(2)));
        assert_eq!(format!("{}", list), "(0 1 . 2)");
    }
}
