pub mod num;

use crate::env::Env;
use crate::eval::{EvalError, EvalResult};
use crate::expr::Expr;

fn check_arity(proc_name: &str, args: &Expr, arity: u32) -> Result<(), EvalError> {
    let mut arg_count = 0_u32;
    let mut args = args;
    loop {
        match args {
            Expr::List(cons) => {
                if let Some(cons) = cons {
                    arg_count += 1;
                    args = &cons.cdr;
                } else {
                    break;
                }
            }
            _ => arg_count += 1,
        }
    }
    if arg_count == arity {
        Ok(())
    } else {
        Err(format!("{} expects {} args!", proc_name, arity))
    }
}

pub fn quote(args: &Expr, _env: &Env) -> EvalResult {
    check_arity("quote", &args, 1)?;

    match args {
        Expr::List(Some(cons)) => Ok(cons.car.as_ref().clone()),
        _ => Err("quote requires a non-empty list.".to_string()),
    }
}
