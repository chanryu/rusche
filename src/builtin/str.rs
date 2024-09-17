use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::get_exact_one_arg;

pub fn is_str(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    if let Expr::Str(_) = eval(get_exact_one_arg(proc_name, args)?, env)? {
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}
