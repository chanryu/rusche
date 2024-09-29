use rusche::{
    eval::{eval, eval_src, EvalContext, EvalResult},
    expr::{Expr, NIL},
    list::List,
};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("print", print);
    context.env.define_native_proc("read", read);

    eval_src(
        r#"
            (define (read-num) (num-parse (read)))
            (define (println *args) (print *args "\n"))
            "#,
        context,
    )
    .expect("Failed to load io procedures");
}

fn print(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }
    Ok(Expr::from(input.trim()))
}
