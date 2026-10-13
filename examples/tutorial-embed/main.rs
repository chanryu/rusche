//! Companion example for docs/tutorials/embedding.md

use rusche::{
    eval, eval_into_proc, utils::get_exact_3_args, EvalContext, EvalResult, Evaluator, Expr, List,
};

fn apply2(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (proc_expr, a_expr, b_expr) = get_exact_3_args(proc_name, args)?;
    let proc = eval_into_proc(proc_name, proc_expr, context)?;
    let a = eval(a_expr, context)?;
    let b = eval(b_expr, context)?;
    context.call(&proc, &[a, b])
}

fn main() {
    // One expression via eval_str
    let evaluator = Evaluator::default();
    let result = evaluator.eval_str("(+ 1 (% 9 2))").unwrap();
    assert_eq!(result, Expr::from(2));

    // Inject a host value, then call a built-in
    evaluator.root_env().define("greeting", Expr::from("hello"));
    let result = evaluator
        .eval_str("(str-append greeting \" world\")")
        .unwrap();
    assert_eq!(result, Expr::from("hello world"));

    // Multi-expression script
    evaluator
        .eval_str(
            r#"
        (define n 0)
        (set! n (+ n 1))
        (set! n (+ n 1))
        "#,
        )
        .unwrap();
    assert_eq!(evaluator.eval_str("n").unwrap(), Expr::from(2));

    // Call a procedure with values the host already has.
    evaluator.eval_str("(define (double x) (* x 2))").unwrap();
    let double = evaluator.eval_str("double").unwrap();
    assert_eq!(
        evaluator.call(&double, &[Expr::from(21)]).unwrap(),
        Expr::from(42)
    );

    // A native calls a script procedure the same way.
    evaluator.root_env().define_native_proc("apply2", apply2);
    assert_eq!(
        evaluator
            .eval_str("(apply2 (lambda (a b) (+ a b)) 20 22)")
            .unwrap(),
        Expr::from(42)
    );

    println!("{}", describe(&result));
}

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Bool(b, _) => format!("boolean {b}"),
        Expr::Num(n, _) => format!("number {n}"),
        Expr::Str(s, _) => format!("string {s:?}"),
        Expr::Sym(name, _) => format!("symbol {name}"),
        Expr::List(list, _) => format!("list {list}"),
        Expr::Proc(proc, _) => format!("procedure <{}>", proc.fingerprint()),
        Expr::Foreign(object) => format!("foreign {:p}", object),
    }
}
