mod common;

use common::{eval_str, parse_single_expr};
use rusp::env::Env;
use rusp::eval::eval;

#[test]
fn test_car() {
    assert_eq!(eval_str("(car '(1 2))"), "1");
}

#[test]
fn test_cdr() {
    assert_eq!(eval_str("(cdr '(1 2))"), "(2)");
}

#[test]
fn test_cond() {
    assert_eq!(eval_str("(cond (#t  0) (#t  1))"), "0");
    assert_eq!(eval_str("(cond (#t  0) (nil 1))"), "0");
    assert_eq!(eval_str("(cond (nil 0) (#t  1))"), "1");
    assert_eq!(eval_str("(cond (nil 0) (nil 1))"), "()");
}

#[test]
fn test_define() {
    let env = Env::new_root_env();
    let expr = parse_single_expr("(define (do-math x y) (- (* x 2) y))");
    let _ = eval(&expr, &env).unwrap();
    let result = eval(&parse_single_expr("(do-math 50 1)"), &env).unwrap();
    assert_eq!(result.to_string(), "99");
}

#[test]
fn test_eval() {
    assert_eq!(eval_str("(eval '(+ 1 2))"), "3");
}

#[test]
fn test_lambda() {
    assert_eq!(eval_str("((lambda (x) (* x 2)) 5)"), "10");
}

#[test]
fn test_quote() {
    assert_eq!(eval_str("'1"), "1");
    assert_eq!(eval_str("'(1)"), "(1)");
    assert_eq!(eval_str("'(1 2)"), "(1 2)");
    assert_eq!(eval_str("'(1 2 (3))"), "(1 2 (3))");
}

#[test]
fn test_quasiquote() {
    // `(0 1 2) => (0 1 2)
    assert_eq!(eval_str("`(0 1 2)"), "(0 1 2)");

    // `(0 ,(+ 1 2) 4) => (0 3 4)
    assert_eq!(eval_str("`(0 ,(+ 1 2) 4)"), "(0 3 4)");

    // `(0 ,@'(1 2 (3 4)) 5) => (0 1 2 (3 4) 5)
    assert_eq!(eval_str("`(0 ,@'(1 2 (3 4)) 5)"), "(0 1 2 (3 4) 5)");
}
