use rand::Rng;
use rusche::{
    arity_error, cons, eval, get_exact_1_arg, ErrorKind, EvalContext, EvalError, EvalResult,
    Evaluator, Expr, Extension, List,
};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::host::Host;

/// `getenv`, `clock`, `random`, and `command-line`.
pub struct SysProcs(pub Rc<Host>);

impl Extension for SysProcs {
    fn install(&self, evaluator: &Evaluator) {
        let env = evaluator.root_env();
        env.define_native_proc("getenv", getenv);
        env.define_native_proc("clock", clock);
        env.define_native_proc("random", random);
        let host = Rc::clone(&self.0);
        env.define_native_proc("command-line", move |proc_name, args, context| {
            command_line(&host, proc_name, args, context)
        });
    }
}

fn getenv(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let value = eval(arg, context)?;
    let Some(name) = value.as_str() else {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: expected a string, got `{value}`"),
        )
        .with_span(arg.span()));
    };
    match std::env::var(name) {
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

fn command_line(host: &Host, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    let mut list = List::Nil;
    for arg in host.command_line().iter().rev() {
        list = cons(Expr::from(arg.as_str()), list);
    }
    Ok(list.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::Evaluator;

    #[test]
    fn command_line_reads_captured_host_state() {
        let host = Host::new(vec!["rusche".into(), "script.rsc".into(), "a".into()]);
        let evaluator = Evaluator::with_builtin();
        evaluator.install(SysProcs(host));
        assert_eq!(
            evaluator.eval_str("(command-line)").unwrap().to_string(),
            r#"("rusche" "script.rsc" "a")"#
        );
    }
}
