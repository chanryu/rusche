mod common;

use common::EvalToStr;
use rusche::eval::Evaluator;

fn eval_str(src: &str) -> String {
    Evaluator::with_prelude().eval_to_str(src)
}

#[test]
fn test_true_false() {
    assert_eq!(eval_str("true"), "true");
    assert_eq!(eval_str("false"), "false");

    // Literals are booleans, not symbols or numbers.
    assert_eq!(eval_str("'true"), "true");
    assert_eq!(eval_str("(eq? true true)"), "true");
    assert_eq!(eval_str("(eq? true 1)"), "false");
    assert_eq!(eval_str("(sym? true)"), "false");
    assert_eq!(eval_str("(num? true)"), "false");
    assert_eq!(eval_str("(atom? true)"), "true");
    assert!(Evaluator::with_prelude().eval_str("(+ true 1)").is_err());
    assert!(Evaluator::with_prelude().eval_str("(define true 1)").is_err());

    // `#t` is an ordinary (undefined) symbol in the core language.
    assert!(eval_str("#t").starts_with("Err:"));
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
    assert_eq!(eval_str("(if true 123 456)"), "123");
    assert_eq!(eval_str("(if false 123 456)"), "456");
    assert!(eval_str("(if 1 (+ 1 2) (+ 3 4))").starts_with("Err:"));
    assert!(eval_str("(if '() (+ 1 2) (+ 3 4))").starts_with("Err:"));
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
    assert_eq!(eval_str("(> 2 1)"), "true");
    assert_eq!(eval_str("(> 1 2)"), "false");
    assert_eq!(eval_str("(> 1 1)"), "false");
    assert_eq!(eval_str("(apply > '(2 1))"), "true");
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
    assert_eq!(eval_str("(and false false)"), "false");
    assert_eq!(eval_str("(and false true)"), "false");
    assert_eq!(eval_str("(and true false)"), "false");
    assert_eq!(eval_str("(and true true)"), "true");

    assert_eq!(eval_str("(or false false)"), "false");
    assert_eq!(eval_str("(or false true)"), "true");
    assert_eq!(eval_str("(or true false)"), "true");
    assert_eq!(eval_str("(or true true)"), "true");

    assert_eq!(eval_str("(not false)"), "true");
    assert_eq!(eval_str("(not true)"), "false");

    // variadic; always returns a boolean
    assert_eq!(eval_str("(and)"), "true");
    assert_eq!(eval_str("(and true true true)"), "true");
    assert_eq!(eval_str("(and true false true)"), "false");
    assert_eq!(eval_str("(or)"), "false");
    assert_eq!(eval_str("(or false true true)"), "true");
    assert_eq!(eval_str("(or false false)"), "false");

    // Non-boolean operands are an error under strict conditions.
    assert!(eval_str("(and 1 2 3)").starts_with("Err:"));
    assert!(eval_str("(or false 2 3)").starts_with("Err:"));

    // short-circuit: the second operand would error if evaluated
    assert_eq!(eval_str("(and false (car '()))"), "false");
    assert_eq!(eval_str("(or true (car '()))"), "true");
    assert_eq!(
        eval_str("(let ((lst '())) (and (not (null? lst)) (eq? (car lst) 1)))"),
        "false"
    );
}

#[test]
fn test_or_else() {
    assert_eq!(eval_str("(or-else false 42)"), "42");
    assert_eq!(eval_str("(or-else 7 42)"), "7");
    assert_eq!(eval_str("(or-else (assoc 'x '((a 1))) 'missing)"), "missing");
    assert_eq!(eval_str("(or-else (assoc 'a '((a 1))) 'missing)"), "(a 1)");
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
    assert_eq!(eval_str("(cond (true  0) (true  1))"), "0");
    assert_eq!(eval_str("(cond (true  0) (false 1))"), "0");
    assert_eq!(eval_str("(cond (false 0) (true  1))"), "1");
    assert_eq!(eval_str("(cond (false 0) (false 1))"), "false");
}

#[test]
fn test_assoc() {
    assert_eq!(eval_str("(assoc 'a '((a 1) (b 2) (c 3)))"), "(a 1)");
    assert_eq!(eval_str("(assoc 'b '((a 1) (b 2) (c 3)))"), "(b 2)");
    assert_eq!(eval_str("(assoc 'x '((a 1) (b 2) (c 3)))"), "false");
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
    assert_eq!(eval_str("(member 'x '(a b c))"), "false");
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
    assert_eq!(eval_str("(< 1 2)"), "true");
    assert_eq!(eval_str("(< 1 2 3)"), "true");
    assert_eq!(eval_str("(< 1 3 2)"), "false");
    assert_eq!(eval_str("(<= 1 1 2)"), "true");
    assert_eq!(eval_str("(> 3 2 1)"), "true");
    assert_eq!(eval_str("(> 3 1 2)"), "false");
    assert_eq!(eval_str("(>= 3 3 1)"), "true");
    assert!(eval_str("(< 1)").starts_with("Err:"));
    assert!(eval_str("(<)").starts_with("Err:"));
}

#[test]
fn test_strict_conditions_report_span() {
    let err = eval_str("(if 1 'a 'b)");
    assert!(err.contains("expected `true` or `false`"), "{err}");
    assert!(err.contains("1:5"), "{err}"); // span of the condition `1`

    let err = eval_str("(not 1)");
    assert!(err.contains("expected `true` or `false`"), "{err}");

    let err = eval_str("(while 1 (+ 1 1))");
    assert!(err.contains("expected `true` or `false`"), "{err}");
}

#[test]
fn test_bool_without_prelude() {
    let e = Evaluator::with_builtin();
    assert_eq!(e.eval_to_str("true"), "true");
    assert_eq!(e.eval_to_str("false"), "false");
    assert_eq!(e.eval_to_str("(if true 1 2)"), "1");
    assert_eq!(e.eval_to_str("(if false 1 2)"), "2");
    assert_eq!(e.eval_to_str("(eq? 1 1)"), "true");
}
