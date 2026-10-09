use rusche::{cons, EvalContext, EvalError, EvalResult, Expr, List};

use crate::host;

pub fn load_sys_procs(context: &EvalContext) {
    context.env.define_native_proc("command-line", command_line);
}

fn command_line(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(EvalError::from(format!(
            "{proc_name} expects no arguments."
        )));
    }
    let mut list = List::Nil;
    for arg in host::command_line().into_iter().rev() {
        list = cons(Expr::from(arg), list);
    }
    Ok(list.into())
}
