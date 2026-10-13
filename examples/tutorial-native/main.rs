//! Companion example for docs/tutorials/native-functions.md

use std::cell::Cell;
use std::rc::Rc;

use rusche::{
    arity_error, tokenize,
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

    // A closure can capture host state. The capture must be `'static`, so share it.
    let count = Rc::new(Cell::new(0));
    let count_for_proc = Rc::clone(&count);
    evaluator
        .root_env()
        .define_native_proc("bump", move |proc_name, args, _| {
            if !args.is_nil() {
                return Err(arity_error(proc_name, 0..=0, args.len()));
            }
            let next = count_for_proc.get() + 1;
            count_for_proc.set(next);
            Ok(Expr::from(next))
        });
    assert_eq!(evaluator.eval_str("(bump)").unwrap(), Expr::from(1));
    assert_eq!(evaluator.eval_str("(bump)").unwrap(), Expr::from(2));
    assert_eq!(count.get(), 2);
}
