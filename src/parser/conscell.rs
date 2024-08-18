#[derive(Debug, PartialEq)]
pub struct Cons {
    car: Box<Expr>,
    cdr: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Nil,
    Number(f64),
    String(String),
    Symbol(String),
    List(Box<Cons>),
}
