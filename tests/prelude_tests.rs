mod common;

use common::EvalToStr;
use rusche::eval::Evaluator;

fn eval_str(src: &str) -> String {
    Evaluator::with_prelude().eval_to_str(src)
}

#[test]
fn test_t_f() {
    assert_eq!(eval_str("#t"), "1");
    assert_eq!(eval_str("#f"), "()");
}

#[test]
fn test_cxxr() {
    // Scheme semantics: c[ad]+r reads right-to-left.
    assert_eq!(eval_str("(caar '((1 2) 3 4))"), "1"); // car of car
    assert_eq!(eval_str("(cadr '((1 2) 3 4))"), "3"); // car of cdr
    assert_eq!(eval_str("(cdar '((1 2) 3 4))"), "(2)"); // cdr of car
    assert_eq!(eval_str("(cddr '((1 2) 3 4))"), "(4)"); // cdr of cdr
}

#[test]
fn test_if() {
    assert_eq!(eval_str("(if #t 123 456)"), "123");
    assert_eq!(eval_str("(if #f 123 456)"), "456");
    assert_eq!(eval_str("(if 1 (+ 1 2) (+ 3 4))"), "3");
    assert_eq!(eval_str("(if '() (+ 1 2) (+ 3 4))"), "7");
}

#[test]
fn test_list() {
    assert_eq!(eval_str("(list)"), "()");
    assert_eq!(eval_str("(list 1)"), "(1)");
    assert_eq!(eval_str("(list 1 2 3)"), "(1 2 3)");
    assert_eq!(eval_str("(list 1 '(2 3))"), "(1 (2 3))");
    // `list` is a procedure, so it can be passed around
    assert_eq!(eval_str("(map list '(1 2))"), "((1) (2))");
    assert_eq!(eval_str("(apply list '(a b))"), "(a b)");
}

#[test]
fn test_map() {
    assert_eq!(eval_str("(map (lambda (x) (* x 2)) '(1 2 3))"), "(2 4 6)");
}

#[test]
fn test_greater() {
    assert_eq!(eval_str("(> 2 1)"), "1");
    assert_eq!(eval_str("(> 1 2)"), "()");
    assert_eq!(eval_str("(> 1 1)"), "()");
    assert_eq!(eval_str("(apply > '(2 1))"), "1");
}

#[test]
fn test_let() {
    let evaluator = Evaluator::with_prelude();
    let context = evaluator.context();

    assert_eq!(context.env.lookup("x"), None);
    assert_eq!(context.eval_to_str("(let ((x 2)) (+ x 3))"), "5");
    assert_eq!(context.env.lookup("x"), None);
}

#[test]
fn test_and_or_not() {
    assert_eq!(eval_str("(and #f #f)"), "()");
    assert_eq!(eval_str("(and #f #t)"), "()");
    assert_eq!(eval_str("(and #t #f)"), "()");
    assert_eq!(eval_str("(and #t #t)"), "1");

    assert_eq!(eval_str("(or #f #f)"), "()");
    assert_eq!(eval_str("(or #f #t)"), "1");
    assert_eq!(eval_str("(or #t #f)"), "1");
    assert_eq!(eval_str("(or #t #t)"), "1");

    assert_eq!(eval_str("(not #f)"), "1");
    assert_eq!(eval_str("(not #t)"), "()");

    // variadic
    assert_eq!(eval_str("(and)"), "1");
    assert_eq!(eval_str("(and 1 2 3)"), "3");
    assert_eq!(eval_str("(and 1 #f 3)"), "()");
    assert_eq!(eval_str("(or)"), "()");
    assert_eq!(eval_str("(or #f 2 3)"), "2");
    assert_eq!(eval_str("(or #f #f)"), "()");

    // short-circuit: the second operand would error if evaluated
    assert_eq!(eval_str("(and #f (car '()))"), "()");
    assert_eq!(eval_str("(or #t (car '()))"), "1");
    assert_eq!(
        eval_str("(let ((lst '())) (and (not (null? lst)) (eq? (car lst) 1)))"),
        "()"
    );
}

#[test]
fn test_append() {
    assert_eq!(eval_str("(append '() '(1))"), "(1)");
    assert_eq!(eval_str("(append '(1) '(2))"), "(1 2)");
    assert_eq!(eval_str("(append '(1 2 3) '(4))"), "(1 2 3 4)");
    assert_eq!(eval_str("(append '(1 2 3) '(4 5 6))"), "(1 2 3 4 5 6)");
}

#[test]
fn test_cond() {
    assert_eq!(eval_str("(cond ('t  0) ('t  1))"), "0");
    assert_eq!(eval_str("(cond ('t  0) ('() 1))"), "0");
    assert_eq!(eval_str("(cond ('() 0) ('t  1))"), "1");
    assert_eq!(eval_str("(cond ('() 0) ('() 1))"), "()");
}

#[test]
fn test_assoc() {
    assert_eq!(eval_str("(assoc 'a '((a 1) (b 2) (c 3)))"), "(a 1)");
    assert_eq!(eval_str("(assoc 'b '((a 1) (b 2) (c 3)))"), "(b 2)");
    assert_eq!(eval_str("(assoc 'x '((a 1) (b 2) (c 3)))"), "()");
}

#[test]
fn test_reverse() {
    assert_eq!(eval_str("(reverse '(a b c d))"), "(d c b a)");
}

#[test]
fn test_length_filter_fold_member() {
    assert_eq!(eval_str("(length '())"), "0");
    assert_eq!(eval_str("(length '(a b c))"), "3");
    assert_eq!(
        eval_str("(filter (lambda (x) (< x 3)) '(1 2 3 4))"),
        "(1 2)"
    );
    assert_eq!(eval_str("(fold + 0 '(1 2 3))"), "6");
    assert_eq!(
        eval_str("(fold (lambda (acc x) (cons x acc)) '() '(a b c))"),
        "(c b a)"
    );
    assert_eq!(eval_str("(member 'b '(a b c))"), "(b c)");
    assert_eq!(eval_str("(member 'x '(a b c))"), "()");
}

#[test]
fn test_let_star() {
    assert_eq!(eval_str("(let* ((x 1) (y (+ x 2))) y)"), "3");
    assert_eq!(eval_str("(let* () 42)"), "42");
    // Shadowing: inner x uses outer x
    assert_eq!(eval_str("(let* ((x 1) (x (+ x 10))) x)"), "11");
}

#[test]
fn test_abs_min_max() {
    assert_eq!(eval_str("(abs -3)"), "3");
    assert_eq!(eval_str("(abs 3)"), "3");
    assert_eq!(eval_str("(min 3)"), "3");
    assert_eq!(eval_str("(min 3 1 2)"), "1");
    assert_eq!(eval_str("(max 3)"), "3");
    assert_eq!(eval_str("(max 3 1 2)"), "3");
}

#[test]
fn test_variadic_comparisons() {
    assert_eq!(eval_str("(< 1 2)"), "1");
    assert_eq!(eval_str("(< 1 2 3)"), "1");
    assert_eq!(eval_str("(< 1 3 2)"), "()");
    assert_eq!(eval_str("(<= 1 1 2)"), "1");
    assert_eq!(eval_str("(> 3 2 1)"), "1");
    assert_eq!(eval_str("(> 3 1 2)"), "()");
    assert_eq!(eval_str("(>= 3 3 1)"), "1");
    assert!(eval_str("(< 1)").starts_with("Err:"));
    assert!(eval_str("(<)").starts_with("Err:"));
}
