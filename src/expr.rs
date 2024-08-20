use crate::env::Env;
use crate::eval::EvalResult;

#[derive(Debug, PartialEq)]
pub struct Cons {
    pub car: Expr,
    pub cdr: Expr,
}

pub type Func = fn(expr: &Expr, env: &Env) -> EvalResult;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Nil,
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Func),
    List(Box<Cons>),
}

impl Expr {
    #[cfg(test)]
    pub fn is_atom(&self) -> bool {
        if let Self::List(_) = self {
            false
        } else {
            true
        }
    }

    pub fn new_sym(text: &str) -> Self {
        Self::Sym(String::from(text))
    }

    pub fn new_list(car: Expr, cdr: Expr) -> Self {
        Self::List(Box::new(Cons { car, cdr }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_atom() {
        assert!(Expr::Nil.is_atom());
        assert!(!Expr::new_list(Expr::Nil, Expr::Nil).is_atom());
    }
}
