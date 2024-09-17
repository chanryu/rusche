use std::rc::Rc;

use rusp::{
    env::Env,
    eval::{eval, EvalResult},
    expr::{Expr, NIL},
    list::List,
};

pub fn display(_: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    for (index, expr) in args.iter().enumerate() {
        if index > 0 {
            print!(" ");
        }
        match eval(expr, env)? {
            Expr::Str(text) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    Ok(NIL)
}
