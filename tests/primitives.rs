mod common;

use common::test_eval;
use rusp::expr::Expr;

#[test]
fn test_car() {
    assert_eq!(test_eval("(car '(0 1))"), Ok(Expr::Num(0_f64)));
}

#[test]
fn test_cond() {
    assert_eq!(test_eval("(cond (t 0) (f 1))"), Ok(Expr::Num(0_f64)));
    assert_eq!(test_eval("(cond (f 0) (t 1))"), Ok(Expr::Num(1_f64)));
}
