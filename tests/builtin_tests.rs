mod common;

use common::EvalToStr;
use rusche::eval::{EvalContext, Evaluator};

fn eval_str(src: &str) -> String {
    Evaluator::with_prelude().eval_to_str(src)
}

#[test]
fn test_car() {
    assert_eq!(eval_str("(car '(1 2))"), "1");
}

#[test]
fn test_cdr() {
    assert_eq!(eval_str("(cdr '(1 2))"), "(2)");
}

#[test]
fn test_define_variable() {
    let e = Evaluator::with_builtin();
    let outer_context = e.context();
    let _ = outer_context.eval_to_str("(define x 1)");
    assert_eq!(outer_context.eval_to_str("x"), "1");
    let _ = outer_context.eval_to_str("(set! x 2)");
    assert_eq!(outer_context.eval_to_str("x"), "2");

    let inner_context = EvalContext::derive_from(&outer_context);

    let _ = inner_context.eval_to_str("(define y 100)");
    assert_eq!(inner_context.eval_to_str("y"), "100");
    let _ = inner_context.eval_to_str("(set! y 200)");
    assert_eq!(inner_context.eval_to_str("y"), "200");

    assert_eq!(inner_context.eval_to_str("x"), "2");
    assert!(outer_context.eval_to_str("y").starts_with("Err:"));
}

#[test]
fn test_define_lambda() {
    let e = Evaluator::with_builtin();
    let _ = e.eval_to_str("(define (do-math x y) (num-subtract (num-multiply x 2) y))");
    assert_eq!(e.eval_to_str("(do-math 50 1)"), "99");
}

#[test]
fn test_eval() {
    assert_eq!(eval_str("(eval '(num-add 1 2))"), "3");
}

#[test]
fn test_if() {
    assert_eq!(eval_str("(if 't 1)"), "1");
    assert_eq!(eval_str("(if 't 1 2)"), "1");

    assert_eq!(eval_str("(if '() 1)"), "()");
    assert_eq!(eval_str("(if '() 1 2)"), "2");
}

#[test]
fn test_lambda() {
    assert_eq!(eval_str("((lambda (x) (num-multiply x 2)) 5)"), "10");
}

#[test]
fn test_set() {
    let e = Evaluator::with_builtin();
    let outer_context = e.context();
    let inner_context = EvalContext::derive_from(&outer_context);

    let _ = outer_context.eval_to_str("(define x 1)");
    assert_eq!(outer_context.eval_to_str("x"), "1");
    let _ = outer_context.eval_to_str("(set! x 2)");
    assert_eq!(outer_context.eval_to_str("x"), "2");

    let _ = inner_context.eval_to_str("(set! x 3)");
    assert_eq!(inner_context.eval_to_str("x"), "3");
    assert_eq!(outer_context.eval_to_str("x"), "3");
}
