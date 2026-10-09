//! Companion example for docs/tutorials/embedding.md

use rusche::{Evaluator, Expr};

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

    println!("{}", describe(&result));
}

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Num(n, _) => format!("number {n}"),
        Expr::Str(s, _) => format!("string {s:?}"),
        Expr::Sym(name, _) => format!("symbol {name}"),
        Expr::List(list, _) => format!("list {list}"),
        Expr::Proc(proc, _) => format!("procedure <{}>", proc.fingerprint()),
        Expr::Foreign(object) => format!("foreign {:p}", object),
    }
}
