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
    assert_eq!(eval_str("(caar '((1 2) 3 4))"), "1");
    assert_eq!(eval_str("(cadr '((1 2) 3 4))"), "(2)");
    assert_eq!(eval_str("(cdar '((1 2) 3 4))"), "3");
    assert_eq!(eval_str("(cddr '((1 2) 3 4))"), "(4)");
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
}

#[test]
fn test_map() {
    assert_eq!(eval_str("(map (lambda (x) (* x 2)) '(1 2 3))"), "(2 4 6)");
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
fn test_pair() {
    assert_eq!(
        eval_str(
            r#"(pair '(1 2 3)
                     '("one" "two" "three"))
            "#
        ),
        r#"((1 "one") (2 "two") (3 "three"))"#,
    );

    assert_eq!(eval_str("(pair '(1 2 3 4) '(5 6))"), "((1 5) (2 6))",);
}

#[test]
fn test_assoc() {
    assert_eq!(eval_str("(assoc 'a '((a 1) (b 2) (c 3)))"), "(a 1)");
    assert_eq!(eval_str("(assoc 'b '((a 1) (b 2) (c 3)))"), "(b 2)");
    assert_eq!(eval_str("(assoc 'x '((a 1) (b 2) (c 3)))"), "()");
}

#[test]
fn test_subst() {
    assert_eq!(eval_str("(subst 'a 'b '(a b c b))"), "(a a c a)");
}

#[test]
fn test_reverse() {
    assert_eq!(eval_str("(reverse '(a b c d))"), "(d c b a)");
}
