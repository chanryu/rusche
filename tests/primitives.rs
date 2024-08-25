mod common;

use common::eval;
use rusp::expr::Expr;

#[test]
fn test_cond() {
    assert_eq!(eval("(cond (t 0) (f 1))"), Ok(Expr::Num(0_f64)));
    assert_eq!(eval("(cond (f 0) (t 1))"), Ok(Expr::Num(1_f64)));
}

#[test]
fn test_quote() {
    assert_eq!(eval("'1"), Ok(Expr::Num(1_f64)));
}
