use rusche::{eval, EvalContext, EvalError, EvalResult, Expr, List, NIL};
use std::io::Write;

use crate::host;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("display", display);
    context.env.define_native_proc("newline", newline);
    context.env.define_native_proc("read", read);
}

/// `(display expr ...)` -- prints each argument; strings are printed without quotes.
fn display(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => {
                print!("{}", text);
                host::note_output(&text);
            }
            expr => {
                let text = expr.to_string();
                print!("{}", text);
                host::note_output(&text);
            }
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn newline(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(EvalError::from(format!(
            "{proc_name} expects no arguments."
        )));
    }
    println!();
    host::note_output("\n");
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => Ok(NIL),
        Ok(_) => Ok(input.trim_end_matches(['\r', '\n']).to_string().into()),
        Err(error) => Err(EvalError::from(format!("Error reading input: {}", error))),
    }
}
