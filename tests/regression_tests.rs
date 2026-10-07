//! One test per finding of the October 2026 bug review. Each test is a reproduction that
//! crashed, hung, or returned a wrong result before the fix.

mod common;

use std::{cell::RefCell, rc::Rc};

use common::EvalToStr;
use rusche::{tokenize, Evaluator, Expr, Parser, Span};

fn eval_str(src: &str) -> String {
    Evaluator::with_prelude().eval_to_str(src)
}

/// Evaluates every top-level form in `src` and returns the last result.
fn eval_all(evaluator: &Evaluator, src: &str) -> Result<Expr, rusche::EvalError> {
    let mut parser = Parser::with_tokens(tokenize(src, None).unwrap());
    let mut last = Ok(rusche::NIL);
    while let Some(expr) = parser.parse().unwrap() {
        last = evaluator.eval(&expr);
    }
    last
}

// B1 -- `eq?` on two native procedures used to recurse until the stack overflowed.
#[test]
fn eq_on_native_procs_terminates() {
    assert_eq!(eval_str("(eq? car car)"), "1");
    assert_eq!(eval_str("(eq? car cdr)"), "()");
    assert_eq!(eval_str("(eq? car (lambda (x) x))"), "()");
}

// B2 -- unbounded recursion aborted the process; now it is an error.
#[test]
fn deep_recursion_is_an_error_not_a_crash() {
    let e = Evaluator::with_prelude();
    e.set_max_call_depth(100);
    assert_eq!(e.max_call_depth(), 100);

    let _ = e.eval_to_str("(define (count n) (if (= n 0) 0 (+ 1 (count (- n 1)))))");
    assert_eq!(e.eval_to_str("(count 10)"), "10");

    let err = e.eval_to_str("(count 1000)");
    assert!(err.contains("Maximum call depth (100) exceeded"), "{err}");

    // the depth counter is restored after the error
    assert_eq!(e.eval_to_str("(count 10)"), "10");

    // tail calls do not consume depth
    let _ = e.eval_to_str("(define (loop n) (if (= n 0) 'done (loop (- n 1))))");
    assert_eq!(e.eval_to_str("(loop 100000)"), "done");
}

// B3 -- a macro expansion mixing call-site and definition-site spans produced an inverted
// span, which panicked in debug builds.
#[test]
fn macro_expansion_with_unordered_spans_does_not_panic() {
    let e = Evaluator::with_prelude();
    let src = "(defmacro (m x) `(println (car ,x 1)))\n\
               (define lst '(1))\n\
               (define (println x) x)\n\
               (m lst)";
    let err = eval_all(&e, src).unwrap_err();
    assert!(err.message.contains("car"), "{}", err.message);
    if let Some(Span { begin, end }) = err.span {
        assert!(begin < end, "span must be ordered: {begin:?}..{end:?}");
    }
}

// B4 -- dropping the evaluator while the host still held a closure tripped a debug assert.
#[test]
fn dropping_evaluator_with_host_held_closure_is_fine() {
    let e = Evaluator::with_prelude();
    let closure = eval_all(&e, "(define (f) (define y 1) (lambda () y)) (f)").unwrap();
    drop(e);
    assert!(matches!(closure, Expr::Proc(..)));
}

// B5 -- closures reachable only through a list had their environment collected.
#[test]
fn gc_keeps_closures_inside_lists_alive() {
    let e = Evaluator::with_prelude();
    let src = "(define (make-counter) (define n 0) (lambda () (set! n (+ n 1)) n))
               (define counters (list (make-counter) (make-counter)))";
    eval_all(&e, src).unwrap();

    assert_eq!(e.eval_to_str("((car counters))"), "1");
    e.collect_garbage();
    assert_eq!(e.eval_to_str("((car counters))"), "2");
    assert_eq!(e.eval_to_str("((cadr counters))"), "1");
}

// B5 -- closures inside a Foreign object survive when a tracer is registered.
#[test]
fn gc_traces_foreign_objects_with_registered_tracer() {
    type ExprVec = RefCell<Vec<Expr>>;

    let e = Evaluator::with_prelude();
    e.register_foreign_tracer::<ExprVec>(|vec, trace| vec.borrow().iter().for_each(trace));

    let counter = eval_all(
        &e,
        "(define (make-counter) (define n 0) (lambda () (set! n (+ n 1)) n)) (make-counter)",
    )
    .unwrap();
    let held: Rc<ExprVec> = Rc::new(RefCell::new(vec![counter]));
    e.root_env().define("box", Expr::Foreign(held.clone()));

    e.collect_garbage();

    let counter = held.borrow()[0].clone();
    e.root_env().define("counter", counter);
    assert_eq!(e.eval_to_str("(counter)"), "1");
}

// B5 -- the value returned by `eval` is a GC root during automatic collection.
#[test]
fn auto_gc_keeps_returned_value_alive() {
    let e = Evaluator::with_prelude();
    e.set_gc_threshold(Some(1)); // collect after every top-level eval
    let counter = eval_all(
        &e,
        "(define (make-counter) (define n 0) (lambda () (set! n (+ n 1)) n)) (make-counter)",
    )
    .unwrap();
    e.root_env().define("counter", counter);
    assert_eq!(e.eval_to_str("(counter)"), "1");
    assert_eq!(e.eval_to_str("(counter)"), "2");
}

// B6 -- the environment registry grew without bound; dead entries are now pruned and
// cyclic garbage is collected automatically.
#[test]
fn environment_registry_does_not_grow_without_bound() {
    let e = Evaluator::with_prelude();
    eval_all(&e, "(define i 0) (while (< i 5000) (set! i (+ i 1)))").unwrap();
    // Per-iteration environments are freed by reference counting and pruned from the
    // registry; the only cyclic garbage is the single `while` wrapper (its env holds the
    // `loop` closure, which captures that env).
    assert!(e.count_unreachable_envs() <= 1);
    e.collect_garbage();
    assert_eq!(e.count_unreachable_envs(), 0);

    // Cyclic garbage (dropped closures) is reclaimed once the threshold is reached.
    e.set_gc_threshold(Some(50));
    let src = "(define (make) (define n 0) (lambda () n))
               (define j 0)
               (while (< j 500) (make) (set! j (+ j 1)))";
    eval_all(&e, src).unwrap();
    assert!(e.count_unreachable_envs() < 50);
}

// B7 -- `(cond ... (else x))` tried to call `x`.
#[test]
fn cond_else_clause() {
    assert_eq!(eval_str("(cond (#f 1) (else 2))"), "2");
    assert_eq!(eval_str("(cond (#f 1) (else 2 3))"), "3");
    assert_eq!(eval_str("(cond (#t 1) (else 2))"), "1");
}

// B8 -- `apply` re-evaluated already evaluated arguments.
#[test]
fn apply_does_not_reevaluate_arguments() {
    assert_eq!(eval_str("(apply car (list (list 1 2)))"), "1");
    assert_eq!(eval_str("(apply + '(1 2 3))"), "6");
    assert_eq!(
        eval_str("(apply (lambda (a b) (cons a b)) (list 'x '(y)))"),
        "(x y)"
    );
    assert_eq!(eval_str("(apply list (list 'a 'b))"), "(a b)");
    assert!(eval_str("(apply 1 '())").starts_with("Err:"));
    assert!(eval_str("(apply car 1)").starts_with("Err:"));
}

// B9 -- covered by `test_cxxr` in prelude_tests.rs; `let` must still work after the swap.
#[test]
fn let_after_cadr_fix() {
    assert_eq!(eval_str("(let ((a 1) (b 2)) (+ a b))"), "3");
}

// B10 -- covered by `test_and_or_not` in prelude_tests.rs.

// B11 -- `set!` on an undefined variable silently succeeded.
#[test]
fn set_on_undefined_variable_is_an_error() {
    let err = eval_str("(set! nope 1)");
    assert!(err.contains("`nope` is not defined"), "{err}");
    assert_eq!(eval_str("(begin (define x 1) (set! x 2) x)"), "2");
}

// B12 -- a variadic parameter that is not last silently dropped the later parameters.
#[test]
fn variadic_parameter_must_be_last() {
    let err = eval_str("(define (f *a b) b)");
    assert!(err.contains("must be the last parameter"), "{err}");
    assert_eq!(eval_str("((lambda (a *rest) rest) 1 2 3)"), "(2 3)");
}

// B13 -- `while` leaked a `loop` binding into the enclosing scope.
#[test]
fn while_does_not_leak_loop_binding() {
    let e = Evaluator::with_prelude();
    eval_all(&e, "(define i 0) (while (< i 3) (set! i (+ i 1)))").unwrap();
    assert_eq!(e.eval_to_str("i"), "3");
    assert!(e.root_env().lookup("loop").is_none());

    // nested loops
    let src = "(define total 0)
               (define a 0)
               (while (< a 3)
                 (define b 0)
                 (while (< b 2) (set! total (+ total 1)) (set! b (+ b 1)))
                 (set! a (+ a 1)))";
    eval_all(&e, src).unwrap();
    assert_eq!(e.eval_to_str("total"), "6");
}

// B14 -- errors raised inside prelude code carried prelude source locations, which the CLI
// rendered against the user's file.
#[test]
fn prelude_errors_point_at_user_code() {
    let e = Evaluator::with_prelude();
    let src =
        "(define x 1)\n(define y 2)\n(define z 3)\n(define w 4)\n(define v 5)\n(map 1 '(1 2))";
    let err = eval_all(&e, src).unwrap_err();
    let span = err.span.expect("error should have a span");
    assert_eq!(
        span.begin.line, 5,
        "span should be on the user's line: {span:?}"
    );
}

// B15 -- closures created via `(define f (lambda ...))` / `defun` were anonymous.
#[test]
fn defined_lambdas_are_named() {
    let e = Evaluator::with_prelude();
    let _ = e.eval_to_str("(defun plus (x y) (+ x y))");
    let err = e.eval_to_str("(plus 1)");
    assert!(err.contains("plus: too few args"), "{err}");

    let _ = e.eval_to_str("(define minus (lambda (x y) (- x y)))");
    let err = e.eval_to_str("(minus 1 2 3)");
    assert!(err.contains("minus: too many args"), "{err}");

    // lambdas and macros now carry a span
    let lambda = eval_all(&e, "(lambda (x) x)").unwrap();
    assert!(lambda.span().is_some());
}

// B16 -- quoted forms had no span.
#[test]
fn quoted_forms_have_spans() {
    let tokens = tokenize("(car 'x 1)", None).unwrap();
    let expr = Parser::with_tokens(tokens).parse().unwrap().unwrap();
    let Expr::List(list, _) = expr else {
        panic!("expected list");
    };
    let quoted = list.iter().nth(1).unwrap();
    let span = quoted.span().expect("quoted form should have a span");
    assert_eq!((span.begin.column, span.end.column), (5, 7));
}

// B17 -- lexer edge cases.
#[test]
fn lexer_edge_cases() {
    assert_eq!(eval_str("(+ .5 1)"), "1.5");
    assert_eq!(eval_str("(+ -.5 1)"), "0.5");
    assert_eq!(eval_str("(let ((b 2)) `(a,b))"), "(a 2)");
    assert_eq!(eval_str("(let ((b '(2 3))) `(a,@b))"), "(a 2 3)");
}

// B18 -- strings were displayed without escaping; foreign objects were never `eq?`.
#[test]
fn display_escapes_strings_and_foreign_is_eq_to_itself() {
    assert_eq!(eval_str(r#"'("a\"b")"#), r#"("a\"b")"#);
    assert_eq!(eval_str(r#"(list "x\ny")"#), r#"("x\ny")"#);

    let e = Evaluator::with_prelude();
    e.root_env().define(
        "v",
        Expr::Foreign(Rc::new(RefCell::new(Vec::<Expr>::new()))),
    );
    e.root_env().define(
        "w",
        Expr::Foreign(Rc::new(RefCell::new(Vec::<Expr>::new()))),
    );
    assert_eq!(e.eval_to_str("(eq? v v)"), "1");
    assert_eq!(e.eval_to_str("(eq? v w)"), "()");
}

// B19 -- prelude list functions are tail-recursive and handle long lists under the
// default call depth limit.
#[test]
fn prelude_list_functions_handle_long_lists() {
    let e = Evaluator::with_prelude();
    let src = "(define (range n) (define (loop i acc) (if (< i 0) acc (loop (- i 1) (cons i acc)))) (loop (- n 1) '()))
               (define big (range 5000))";
    eval_all(&e, src).unwrap();
    assert_eq!(e.eval_to_str("(car (reverse big))"), "4999");
    assert_eq!(
        e.eval_to_str("(car (reverse (map (lambda (x) (* x 2)) big)))"),
        "9998"
    );
    assert_eq!(e.eval_to_str("(car (reverse (append big '(-1))))"), "-1");
}
