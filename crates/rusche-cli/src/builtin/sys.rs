use rand::Rng;
use rusche::{
    arity_error, cons, eval, get_exact_1_arg, ErrorKind, EvalContext, EvalError, EvalResult, Expr,
    List,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::host;

pub fn load_sys_procs(context: &EvalContext) {
    context.env.define_native_proc("getenv", getenv);
    context.env.define_native_proc("clock", clock);
    context.env.define_native_proc("random", random);
    context.env.define_native_proc("command-line", command_line);
}

fn getenv(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let name = match eval(arg, context)? {
        Expr::Str(name, _) => name,
        other => {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: expected a string, got `{other}`"),
            )
            .with_span(arg.span()));
        }
    };
    match std::env::var(&name) {
        Ok(value) => Ok(value.into()),
        Err(_) => Ok(false.into()),
    }
}

fn clock(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    Ok(secs.into())
}

fn random(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    Ok(rand::thread_rng().gen::<f64>().into())
}

fn command_line(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    let mut list = List::Nil;
    for arg in host::command_line().into_iter().rev() {
        list = cons(Expr::from(arg), list);
    }
    Ok(list.into())
}
