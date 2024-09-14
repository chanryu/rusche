mod common;

use common::{eval_str, eval_str_env};
use rusp::env::Env;

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
fn test_cxxxr() {
    assert_eq!(eval_str("(cadar '((1 2 3) 4))"), "2");
    assert_eq!(eval_str("(caddr '((1 2 3) 4))"), "(3)");
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
    let env = Env::with_prelude();

    assert_eq!(env.lookup("x"), None);
    assert_eq!(eval_str_env("(let ((x 2)) (+ x 3))", &env), "5");
    assert_eq!(env.lookup("x"), None);
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
