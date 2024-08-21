use crate::env::Env;
use crate::eval::EvalResult;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Cons {
    pub car: Expr,
    pub cdr: Expr,
}

pub type Func = fn(expr: &Expr, env: &Env) -> EvalResult;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Nil,
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Func),
    List(Box<Cons>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Nil => write!(f, "()"),
            Expr::Num(value) => write!(f, "{}", value),
            Expr::Str(text) => write!(f, "\"{}\"", text), // TODO: escape as control chars
            Expr::Sym(text) => write!(f, "{}", text),
            Expr::Proc(func) => write!(f, "<#proc: {:?}>", func),
            Expr::List(cons) => write_cons(f, cons, true),
        }
    }
}

fn write_cons(f: &mut fmt::Formatter<'_>, cons: &Cons, is_top_level: bool) -> fmt::Result {
    if is_top_level {
        write!(f, "(")?;
    }
    match &cons.cdr {
        Expr::Nil => write!(f, "{}", cons.car),
        Expr::List(inner_cons) => {
            write!(f, "{} ", cons.car)?;
            write_cons(f, &inner_cons, false)
        }
        _ => write!(f, "{} . {}", cons.car, cons.cdr),
    }?;
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}
