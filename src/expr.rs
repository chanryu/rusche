#[derive(Debug, PartialEq)]
pub struct Cons {
    pub car: Expr,
    pub cdr: Expr,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Nil,
    Num(f64),
    Str(String),
    Sym(String),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_atom() {
        assert!(Expr::Nil.is_atom());
        assert!(!Expr::List(Box::new(Cons {
            car: Expr::Nil,
            cdr: Expr::Nil
        }))
        .is_atom());
    }
}
