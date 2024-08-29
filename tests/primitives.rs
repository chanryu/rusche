mod common;

#[macro_use]
extern crate rusp;

use common::eval_expr;
use rusp::{
    expr::shortcuts::num,
    list::{cons, List},
};

#[test]
fn test_cond() {
    assert_eq!(eval_expr("(cond (t 0) (f 1))"), Ok(num(0)));
    assert_eq!(eval_expr("(cond (f 0) (t 1))"), Ok(num(1)));
}

#[test]
fn test_lambda() {
    assert_eq!(eval_expr("((lambda (x) (* x 2)) 5)"), Ok(num(10)));
}

#[test]
fn test_quote() {
    assert_eq!(eval_expr("'1"), Ok(num(1)));
    assert_eq!(eval_expr("'(1)"), Ok(list!(num(1)).into()));
    assert_eq!(eval_expr("'(1 2)"), Ok(list!(num(1), num(2)).into()));
}
