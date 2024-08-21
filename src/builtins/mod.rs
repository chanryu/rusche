pub mod num;

use crate::env::Env;
use crate::eval::{EvalError, EvalResult};
use crate::expr::Expr;

fn check_arity(proc_name: &str, args: &Expr, arity: u32) -> Result<(), EvalError> {
    let mut arg_count = 0_u32;
    let mut args = args;
    loop {
        match args {
            Expr::Nil => break,
            Expr::List(cons) => {
                arg_count += 1;
                args = &cons.cdr;
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

    if let Expr::List(cons) = args {
        Ok(cons.car.clone())
    } else {
        Err(String::from("quote requires a list."))
    }
}
