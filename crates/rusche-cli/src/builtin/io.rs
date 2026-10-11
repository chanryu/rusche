use rusche::{
    eval, eval_source, get_exact_1_arg, ErrorKind, EvalContext, EvalError, EvalResult, Expr, List,
    NIL,
};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::diagnostics::print_pipeline_error;
use crate::host;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("write", write);
    context.env.define_native_proc("read", read);
    context.env.define_native_proc("exit", exit);
    context.env.define_native_proc("load", load);
}

/// `(write expr ...)` -- writes each argument with nothing between them.
/// Strings are raw text; other values use their source form. A newline is `"\n"`.
fn write(_: &str, args: &List, context: &EvalContext) -> EvalResult {
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

    let resolved = resolve_load_path(&path);
    let display = resolved.display().to_string();

    let text = std::fs::read_to_string(&resolved).map_err(|e| {
        EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: failed to read \"{display}\": {e}"),
        )
        .with_span(arg.span())
    })?;

    // Nested `load` calls resolve relative to this file, not the caller.
    let _source = host::enter_source_file(&resolved);
    match eval_source(&text, context, false) {
        Ok(result) => Ok(result),
        Err(error) => {
            // Print the inner file's diagnostic with its own source, then return a terse error
            // so the caller's caret lands on the `(load ...)` form without embedding spans in prose.
            print_pipeline_error(&error, &text, &display);
            Err(EvalError::new(
                ErrorKind::Other,
                format!("{proc_name}: \"{display}\" failed"),
            )
            .with_span(arg.span()))
        }
    }
}

/// Absolute paths are used as given. A relative path is joined onto the directory of the
/// file currently being evaluated, or left for the process working directory when there
/// is none (REPL, `-e`, stdin).
fn resolve_load_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(dir) = host::current_source_dir() {
        dir.join(path)
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::Evaluator;
    use std::io::Write;

    fn with_io() -> Evaluator {
        let evaluator = Evaluator::with_builtin();
        load_io_procs(evaluator.context());
        evaluator
    }

    fn eval_err(evaluator: &Evaluator, src: &str) -> String {
        match evaluator.eval_str(src) {
            Ok(v) => panic!("expected error, got {v}"),
            Err(e) => e.message(),
        }
    }

    #[test]
    fn write_and_load_errors() {
        colored::control::set_override(false);
        let e = with_io();

        assert!(e.eval_str(r#"(write "hi" 1)"#).is_ok());
        assert!(e.eval_str(r#"(write (show "hi"))"#).is_ok());
        assert!(e.eval_str(r#"(write)"#).is_ok());

        assert!(eval_err(&e, "(load 1)").contains("string path"));
        assert!(eval_err(&e, r#"(load "/tmp/rusche-missing-xyz.rsc")"#).contains("failed to read"));

        let dir = std::env::temp_dir().join(format!(
            "rusche-io-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.rsc");
        {
            let mut f = std::fs::File::create(&bad).unwrap();
            writeln!(f, "(car 1)").unwrap();
        }
        let src = format!(r#"(load "{}")"#, bad.display());
        let err = eval_err(&e, &src);
        assert!(err.contains("failed"), "{err}");
        assert!(eval_err(&e, "(exit 999)").contains("0-255"));
        assert!(eval_err(&e, r#"(exit "no")"#).contains("0-255"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
