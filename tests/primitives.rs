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
    assert_eq!(eval_expr("'(1 2 (3))"), "(1 2 (3))");
}

#[test]
fn test_quasiquote() {
    // > (quasiquote (0 1 2))
    // (0 1 2)

    assert_eq!(eval_expr("`(0 1 2)"), "(0 1 2)");

    // > (quasiquote (0 (unquote (+ 1 2)) 4))
    // (0 3 4)

    // > (quasiquote (0 (unquote-splicing (list 1 2)) 4))
    // (0 1 2 4)

    // > (quasiquote (0 (unquote-splicing 1) 4))
    // unquote-splicing: expected argument of type <proper list>;
    // given 1

    // > (quasiquote (0 (unquote-splicing 1)))
    // (0 . 1)
}
