//! Rest parameters are spelled `*name` and must come last: `(a *rest)`.

mod common;

use common::EvalToStr;
use rusche::Evaluator;

fn eval_str(src: &str) -> String {
    Evaluator::with_prelude().eval_to_str(src)
}

#[test]
fn rest_parameter_in_lambda_define_and_defmacro() {
    assert_eq!(eval_str("((lambda (a *rest) rest) 1 2 3)"), "(2 3)");
    assert_eq!(eval_str("((lambda (a *rest) rest) 1)"), "()");
    assert_eq!(eval_str("((lambda (*args) args) 1 2 3)"), "(1 2 3)");
    assert_eq!(eval_str("((lambda (*args) args))"), "()");

    let e = Evaluator::with_prelude();
    let _ = e.eval_to_str("(define (f a *rest) (cons a rest))");
    assert_eq!(e.eval_to_str("(f 1 2 3)"), "(1 2 3)");

    let _ = e.eval_to_str("(defun g (a *rest) rest)");
    assert_eq!(e.eval_to_str("(g 1 2 3)"), "(2 3)");

    let _ = e.eval_to_str("(define (count *xs) (if (null? xs) 0 (+ 1 (apply count (cdr xs)))))");
    assert_eq!(e.eval_to_str("(count 'a 'b 'c)"), "3");

    let _ = e.eval_to_str("(defmacro (first-form *forms) (car forms))");
    assert_eq!(e.eval_to_str("(first-form (+ 1 2) (undefined))"), "3");

    let _ = e.eval_to_str("(defmacro last-form (*forms) (car (reverse forms)))");
    assert_eq!(e.eval_to_str("(last-form (undefined) (+ 1 2))"), "3");

    // arity errors still apply to the fixed part
    let _ = e.eval_to_str("(defmacro (two a b *rest) a)");
    assert!(e.eval_to_str("(two 1)").contains("two: expected 2 arguments, got 1"));
    assert!(eval_str("((lambda (a b) a) 1)").contains("expected 2 arguments, got 1"));
    assert!(eval_str("((lambda (a) a) 1 2)").contains("expected 1 argument, got 2"));
}

#[test]
fn rest_arguments_of_closures_are_evaluated() {
    // Each rest argument is evaluated exactly once, in order, in the caller's environment.
    let e = Evaluator::with_prelude();
    let _ = e.eval_to_str("(define n 0)");
    let _ = e.eval_to_str("(define (tick) (set! n (+ n 1)) n)");
    let _ = e.eval_to_str("(define (f *xs) xs)");
    assert_eq!(e.eval_to_str("(f (tick) (tick) (tick))"), "(1 2 3)");
    assert_eq!(e.eval_to_str("n"), "3");
}

#[test]
fn malformed_rest_parameters_are_errors() {
    for src in [
        "(lambda (*a b) a)",
        "(lambda (*a *b) a)",
        "(lambda (*a*) a)",
        "(lambda (**a) a)",
        "(lambda (a 1) a)",
        "(lambda 1 a)",
        "(lambda)",
        "(define (f *a b) b)",
        "(defmacro m)",
    ] {
        let out = eval_str(src);
        assert!(
            out.contains("not a symbol")
                || out.contains("rest parameter")
                || out.contains("not a valid parameter list")
                || out.contains("expected formal arguments"),
            "{src} => {out}"
        );
    }
}

#[test]
fn bare_symbol_parameter_list_is_an_error() {
    // Scheme's `(lambda args ...)` is dotted-pair syntax in disguise; Rusche spells it `(*args)`.
    for src in ["(lambda args args)", "(defmacro m forms forms)"] {
        let out = eval_str(src);
        assert!(out.contains("not a valid parameter list"), "{src} => {out}");
    }
}

#[test]
fn star_alone_and_star_inside_a_name_are_ordinary_parameters() {
    assert_eq!(eval_str("((lambda (*) *) 7)"), "7");
    assert_eq!(eval_str("((lambda (a*b) a*b) 7)"), "7");
    // `*rest` binds `rest`, so the symbol `*rest` itself is unbound.
    assert!(eval_str("((lambda (*rest) *rest) 1 2)").contains("undefined symbol: `*rest`"));
}

