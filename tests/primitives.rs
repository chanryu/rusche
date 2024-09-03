mod common;

use common::{eval_expr, parse_expr};
use rusp::env::Env;
use rusp::eval::eval;

#[test]
fn test_cond() {
    assert_eq!(eval_expr("(cond (t 0) (f 1))"), "0");
    assert_eq!(eval_expr("(cond (f 0) (t 1))"), "1");
}

#[test]
fn test_define() {
    let env = Env::new_root_env();
    let expr = parse_expr("(define (plus x y) (+ x y))");
    let _ = eval(&expr, &env).unwrap();
    let result = eval(&parse_expr("(plus 99 1)"), &env).unwrap();
    assert_eq!(result.to_string(), "100");
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
    // `(0 1 2) => (0 1 2)
    assert_eq!(eval_expr("`(0 1 2)"), "(0 1 2)");

    // `(0 ,(+ 1 2) 4) => (0 3 4)
    assert_eq!(eval_expr("`(0 ,(+ 1 2) 4)"), "(0 3 4)");

    // `(0 ,@'(1 2 (3 4)) 5) => (0 1 2 (3 4) 5)
    assert_eq!(eval_expr("`(0 ,@'(1 2 (3 4)) 5)"), "(0 1 2 (3 4) 5)");
}
