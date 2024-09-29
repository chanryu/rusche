mod common;

use common::EvalToStr;
use rusche::eval::Evaluator;

#[test]
fn test_quote() {
    let e = Evaluator::with_builtin();
    assert_eq!(e.eval_to_str("'1"), "1");
    assert_eq!(e.eval_to_str("'(1)"), "(1)");
    assert_eq!(e.eval_to_str("'(1 2)"), "(1 2)");
    assert_eq!(e.eval_to_str("'(1 2 (3))"), "(1 2 (3))");
}

#[test]
fn test_quasiquote() {
    let e = Evaluator::with_builtin();

    // `(0 1 2) => (0 1 2)
    assert_eq!(e.eval_to_str("`(0 1 2)"), "(0 1 2)");

    // `(0 ,(+ 1 2) 4) => (0 3 4)
    assert_eq!(e.eval_to_str("`(0 ,(num-add 1 2) 4)"), "(0 3 4)");

    // `(0 ,@'(1 2 (3 4)) 5) => (0 1 2 (3 4) 5)
    assert_eq!(e.eval_to_str("`(0 ,@'(1 2 (3 4)) 5)"), "(0 1 2 (3 4) 5)");
}
