// Let's prove that the demo code in README.md works.

use rusche::{
    utils::{eval_into_num, get_exact_2_args},
    EvalContext, EvalResult, Evaluator, Expr, List,
};

// A native function: Rust code that scripts can call. Arguments arrive
// unevaluated; the helpers in `rusche::utils` evaluate and type-check them.
fn hypot(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (a, b) = get_exact_2_args(proc_name, args)?;
    let a = eval_into_num(proc_name, a, context)?;
    let b = eval_into_num(proc_name, b, context)?;
    Ok(Expr::from(a.hypot(b)))
}

fn main() {
    // Built-ins plus the prelude (`+`, `*`, `sqrt`, `map`, `let`, ...)
    let evaluator = Evaluator::default();

    // Expose host functionality to scripts
    evaluator.root_env().define_native_proc("hypot", hypot);

    // Tokenize, parse, and evaluate every top-level form; the last value is returned
    let result = evaluator
        .eval_str(
            r#"
            (map (lambda (p) (apply hypot p)) '((3 4) (5 12)))
            "#,
        )
        .unwrap();

    println!("{result}"); // (5 13)
    assert_eq!(result, evaluator.eval_str("'(5 13)").unwrap());

    // Lex, parse, and eval failures share one `Error` type with a source span
    let err = evaluator.eval_str("(hypot \"nine\" 4)").unwrap_err();
    println!("{err}"); // 1:8-13: hypot: `"nine"` evaluated to `"nine"`, expected a number
}
