use rusche::{
    arity_error, eval, eval_source, get_exact_1_arg, ErrorKind, EvalContext, EvalError, EvalResult,
    Expr, List, NIL,
};
use std::io::Write;

use crate::diagnostics::print_pipeline_error;
use crate::host;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("display", display);
    context.env.define_native_proc("write", write);
    context.env.define_native_proc("newline", newline);
    context.env.define_native_proc("read", read);
    context.env.define_native_proc("exit", exit);
    context.env.define_native_proc("load", load);
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

/// `(write expr ...)` -- prints each argument using `Display` (strings quoted).
fn write(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    for expr in args.iter() {
        let text = eval(expr, context)?.to_string();
        print!("{}", text);
        host::note_output(&text);
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn newline(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    println!();
    host::note_output("\n");
    Ok(NIL)
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => Ok(false.into()),
        Ok(_) => Ok(input.trim_end_matches(['\r', '\n']).to_string().into()),
        Err(error) => Err(EvalError::new(
            ErrorKind::Other,
            format!("error reading input: {error}"),
        )),
    }
}

fn exit(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let code = if args.is_nil() {
        0
    } else {
        let arg = get_exact_1_arg(proc_name, args)?;
        let value = eval(arg, context)?;
        match value {
            Expr::Num(n, _) if n.fract() == 0.0 && (0.0..=255.0).contains(&n) => n as i32,
            other => {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: exit code must be an integer 0-255, got `{other}`"),
                )
                .with_span(arg.span()));
            }
        }
    };
    let _ = std::io::stdout().flush();
    std::process::exit(code);
}

fn load(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let path = match eval(arg, context)? {
        Expr::Str(path, _) => path,
        other => {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: expected a string path, got `{other}`"),
            )
            .with_span(arg.span()));
        }
    };

    let text = std::fs::read_to_string(&path).map_err(|e| {
        EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: failed to read \"{path}\": {e}"),
        )
        .with_span(arg.span())
    })?;

    match eval_source(&text, context, false) {
        Ok(result) => Ok(result),
        Err(error) => {
            // Print the inner file's diagnostic with its own source, then return a terse error
            // so the caller's caret lands on the `(load ...)` form without embedding spans in prose.
            print_pipeline_error(&error, &text, &path);
            Err(EvalError::new(
                ErrorKind::Other,
                format!("{proc_name}: \"{path}\" failed"),
            )
            .with_span(arg.span()))
        }
    }
}
