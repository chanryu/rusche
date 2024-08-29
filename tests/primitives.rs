mod common;

use common::eval_expr;

#[test]
fn test_cond() {
    assert_eq!(eval_expr("(cond (t 0) (f 1))"), "0");
    assert_eq!(eval_expr("(cond (f 0) (t 1))"), "1");
}

#[test]
fn test_lambda() {
    assert_eq!(eval_expr("((lambda (x) (* x 2)) 5)"), "10");
}

#[test]
fn test_quote() {
    assert_eq!(eval_expr("'1"), "1");
    assert_eq!(eval_expr("'(1)"), "(1)");
    assert_eq!(eval_expr("'(1 2)"), "(1 2)");
}
