//! Companion example for docs/tutorials/native-functions.md

use rusche::{
    tokenize,
    utils::{eval_into_num, get_exact_3_args},
    EvalContext, EvalResult, Evaluator, Expr, List, Parser,
};

fn clamp(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (value_expr, lo_expr, hi_expr) = get_exact_3_args(proc_name, args)?;
    let value = eval_into_num(proc_name, value_expr, context)?;
    let lo = eval_into_num(proc_name, lo_expr, context)?;
    let hi = eval_into_num(proc_name, hi_expr, context)?;

    Ok(Expr::from(value.clamp(lo, hi)))
}

fn main() {
    let evaluator = Evaluator::with_prelude();
    evaluator.root_env().define_native_proc("clamp", clamp);

    let result = evaluator
        .eval(
            &Parser::with_tokens(tokenize("(clamp 15 0 10)", None).unwrap())
                .parse()
                .unwrap()
                .unwrap(),
        )
        .unwrap();

    assert_eq!(result, Expr::from(10));
    println!("{result}");
}
