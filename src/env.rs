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
