use crate::builtins;
use crate::expr::Expr;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Env {
    vars: RefCell<HashMap<String, Expr>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_root_env() -> Self {
        let env = Env::new();

        // lisp primitives
        env.set("define", Expr::Proc(builtins::define));
        env.set("quote", Expr::Proc(builtins::quote));

        // arithmetic operations
        env.set("+", Expr::Proc(builtins::num::add));
        env.set("-", Expr::Proc(builtins::num::minus));
        env.set("*", Expr::Proc(builtins::num::multiply));
        env.set("/", Expr::Proc(builtins::num::divide));

        env
    }

    pub fn get(&self, name: &str) -> Option<Expr> {
        self.vars.borrow().get(name).cloned()
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.vars.borrow_mut().insert(name.to_string(), expr);
    }
}

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Env) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.get(name) {
            Some(Expr::Proc(func)) => Ok(Expr::Proc(func.clone())),
            Some(_) => Err(format!("{} is not a procedure!", name)),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(Some(cons)) => {
            if let Expr::Proc(func) = eval(&cons.car, env)? {
                func(&cons.cdr, env)
            } else {
                Err(format!("{} does not evaluate to a callable.", cons.car))
            }
        }
        _ => Ok(expr.clone()),
    }
}
