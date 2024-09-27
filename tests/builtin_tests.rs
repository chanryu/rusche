mod common;

use common::{eval_str, eval_str_env};
use rusche::eval::{EvalContext, Evaluator};

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
    let evaluator = Evaluator::with_builtin();
    let outer_context = evaluator.context();
    let _ = eval_str_env("(define x 1)", &outer_context);
    assert_eq!(eval_str_env("x", &outer_context), "1");
    let _ = eval_str_env("(set! x 2)", &outer_context);
    assert_eq!(eval_str_env("x", &outer_context), "2");

    let inner_context = EvalContext::derive_from(&outer_context);

    let _ = eval_str_env("(define y 100)", &inner_context);
    assert_eq!(eval_str_env("y", &inner_context), "100");
    let _ = eval_str_env("(set! y 200)", &inner_context);
    assert_eq!(eval_str_env("y", &inner_context), "200");

    assert_eq!(eval_str_env("x", &inner_context), "2");
    assert!(eval_str_env("y", &outer_context).starts_with("Err:"));
}

#[test]
fn test_define_lambda() {
    let evaluator = Evaluator::with_builtin();
    let context = evaluator.context();
    let _ = eval_str_env(
        "(define (do-math x y) (num-subtract (num-multiply x 2) y))",
        context,
    );
    assert_eq!(eval_str_env("(do-math 50 1)", context), "99");
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
    let evaluator = Evaluator::with_builtin();
    let outer_context = evaluator.context();
    let inner_context = EvalContext::derive_from(&outer_context);

    let _ = eval_str_env("(define x 1)", &outer_context);
    assert_eq!(eval_str_env("x", &outer_context), "1");
    let _ = eval_str_env("(set! x 2)", &outer_context);
    assert_eq!(eval_str_env("x", &outer_context), "2");

    let _ = eval_str_env("(set! x 3)", &inner_context);
    assert_eq!(eval_str_env("x", &inner_context), "3");
    assert_eq!(eval_str_env("x", &outer_context), "3");
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
    assert_eq!(eval_str("`(0 ,(num-add 1 2) 4)"), "(0 3 4)");

    // `(0 ,@'(1 2 (3 4)) 5) => (0 1 2 (3 4) 5)
    assert_eq!(eval_str("`(0 ,@'(1 2 (3 4)) 5)"), "(0 1 2 (3 4) 5)");
}
