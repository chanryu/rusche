use crate::env::Env;
use crate::eval::EvalResult;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Cons {
    pub car: Expr,
    pub cdr: Expr,
}

pub type Func = fn(expr: &Expr, env: &Env) -> EvalResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Str(String),
    Sym(String),
    Proc(Func),
    List(Option<Box<Cons>>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(value) => write!(f, "{}", value),
            Expr::Str(text) => write!(f, "\"{}\"", text), // TODO: escape as control chars
            Expr::Sym(text) => write!(f, "{}", text),
            Expr::Proc(func) => write!(f, "<#proc: {:?}>", func),
            Expr::List(None) => write!(f, "()"),
            Expr::List(Some(cons)) => write_cons(f, cons, true),
        }
    }
}

fn write_cons(f: &mut fmt::Formatter<'_>, cons: &Cons, is_top_level: bool) -> fmt::Result {
    if is_top_level {
        write!(f, "(")?;
    }
    match &cons.cdr {
        Expr::List(None) => write!(f, "{}", cons.car),
        Expr::List(Some(inner_cons)) => {
            write!(f, "{} ", cons.car)?;
            write_cons(f, inner_cons, false)
        }
        _ => write!(f, "{} . {}", cons.car, cons.cdr),
    }?;
    if is_top_level {
        write!(f, ")")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_nil() {
        assert_eq!(format!("{}", Expr::List(None)), "()");
    }

    #[test]
    fn test_display_num() {
        assert_eq!(format!("{}", Expr::Num(0_f64)), "0");
        assert_eq!(format!("{}", Expr::Num(1_f64)), "1");
        assert_eq!(format!("{}", Expr::Num(1.2)), "1.2");
        assert_eq!(format!("{}", Expr::Num(2.0)), "2");
    }

    #[test]
    fn test_display_str() {
        assert_eq!(format!("{}", Expr::Str(String::from("str"))), "\"str\"");
    }

    #[test]
    fn test_display_sym() {
        assert_eq!(format!("{}", Expr::Sym(String::from("sym"))), "sym");
    }

    #[test]
    fn test_display_list_1() {
        let list = Expr::List(Some(Box::new(Cons {
            car: Expr::Num(0_f64),
            cdr: Expr::List(None),
        })));
        assert_eq!(format!("{}", list), "(0)");
    }

    #[test]
    fn test_display_list_2() {
        let list = Expr::List(Some(Box::new(Cons {
            car: Expr::Num(0_f64),
            cdr: Expr::List(Some(Box::new(Cons {
                car: Expr::Num(1_f64),
                cdr: Expr::List(Some(Box::new(Cons {
                    car: Expr::Num(2_f64),
                    cdr: Expr::List(None),
                }))),
            }))),
        })));
        assert_eq!(format!("{}", list), "(0 1 2)");
    }

    #[test]
    fn test_display_list_3() {
        let list = Expr::List(Some(Box::new(Cons {
            car: Expr::Num(0_f64),
            cdr: Expr::List(Some(Box::new(Cons {
                car: Expr::Str("str".to_string()),
                cdr: Expr::List(Some(Box::new(Cons {
                    car: Expr::Sym("sym".to_string()),
                    cdr: Expr::List(None),
                }))),
            }))),
        })));
        assert_eq!(format!("{}", list), "(0 \"str\" sym)");
    }

    #[test]
    fn test_display_irregular_list_1() {
        let list = Expr::List(Some(Box::new(Cons {
            car: Expr::Num(0_f64),
            cdr: Expr::Num(1_f64),
        })));
        assert_eq!(format!("{}", list), "(0 . 1)");
    }

    #[test]
    fn test_display_irregular_list_2() {
        let list = Expr::List(Some(Box::new(Cons {
            car: Expr::Num(0_f64),
            cdr: Expr::List(Some(Box::new(Cons {
                car: Expr::Num(1_f64),
                cdr: Expr::Num(2_f64),
            }))),
        })));
        assert_eq!(format!("{}", list), "(0 1 . 2)");
    }
}
