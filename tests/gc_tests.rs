mod common;

use common::EvalToStr;
use rusche::eval::Evaluator;

#[test]
fn test_gc() {
    let e = Evaluator::with_builtin();

    assert_eq!(e.count_unreachable_envs(), 0);

    // a simple code that creates an unreachable env
    let _ = e.eval_to_str(
        r#"
        (define (f)
            (define (g) '())
            g)
        "#,
    );
    assert_eq!(e.count_unreachable_envs(), 0); // no unreachable envs yet

    let _ = e.eval_to_str("(f)");
    assert_eq!(e.count_unreachable_envs(), 1);

    let _ = e.eval_to_str("(f)");
    assert_eq!(e.count_unreachable_envs(), 2);

    let _ = e.eval_to_str("(f)");
    let _ = e.eval_to_str("(f)");
    assert_eq!(e.count_unreachable_envs(), 4);

    e.collect_garbage();
    assert_eq!(e.count_unreachable_envs(), 0);
}
