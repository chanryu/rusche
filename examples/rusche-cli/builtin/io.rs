use rusche::{eval, EvalContext, EvalError, EvalResult, Expr, List, NIL};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("display", display);
    context.env.define_native_proc("newline", newline);
    context.env.define_native_proc("read", read);
}

/// `(display expr ...)` -- prints each argument; strings are printed without quotes.
fn display(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text),
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn newline(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    println!();
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(EvalError::from(format!("Error reading input: {}", error)));
    }
    Ok(input.trim().to_string().into())
}
