// Let's prove that the demo code in README.md works.

use rusche::{
    utils::{eval_into_num, get_exact_1_arg},
    EvalContext, EvalResult, Evaluator, Expr, List,
};

// A native function: Rust code that scripts can call. Arguments arrive
// unevaluated; the helpers in `rusche::utils` evaluate and type-check them.
fn sqrt(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let n = eval_into_num(proc_name, arg, context)?;
    Ok(Expr::from(n.sqrt()))
}

fn main() {
    // Built-ins plus the prelude (`+`, `*`, `map`, `let`, ...)
    let evaluator = Evaluator::default();

    // Expose host functionality to scripts
    evaluator.root_env().define_native_proc("sqrt", sqrt);

    // Tokenize, parse, and evaluate every top-level form; the last value is returned
    let result = evaluator
        .eval_str(
            r#"
            (define (hypot a b) (sqrt (+ (* a a) (* b b))))
            (map (lambda (p) (apply hypot p)) '((3 4) (5 12)))
            "#,
        )
        .unwrap();

    println!("{result}"); // (5 13)
    assert_eq!(result, evaluator.eval_str("'(5 13)").unwrap());

    // Lex, parse, and eval failures share one `Error` type with a source span
    let err = evaluator.eval_str("(sqrt \"nine\")").unwrap_err();
    println!("{err}"); // 1:7-12: sqrt: `"nine"` evaluated to `"nine"`, expected a number
}
