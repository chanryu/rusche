use std::collections::HashMap;

use crate::expr::Expr;

pub type EvalResult = Result<Expr, String>;

pub struct Env {
    vars: HashMap<String, Expr>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Expr> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: &str, expr: Expr) {
        let _ = self.vars.insert(name.to_string(), expr);
    }
}

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Nil => Ok(Expr::Nil),
        Expr::Num(value) => Ok(Expr::Num(value.clone())),
        Expr::Str(text) => Ok(Expr::Str(text.clone())),
        Expr::Sym(text) => match env.get(text) {
            Some(Expr::Proc(func)) => Ok(Expr::Proc(func.clone())),
            Some(_) => Err(format!("{} is not a procedure!", text)),
            None => Err(format!("Undefined symbol: {:?}", text)),
        },
        Expr::Proc(func) => Ok(Expr::Proc(func.clone())),
        Expr::List(cons) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(&cons.cdr, env)
            } else {
                Err(String::from("A Proc is expected."))
            }
        }
    }
}
