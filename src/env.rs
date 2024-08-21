use crate::builtins;
use crate::expr::Expr;
use std::collections::HashMap;

pub struct Env {
    vars: HashMap<String, Expr>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn new_root_env() -> Self {
        let mut env = Env::new();

        // lisp primitives
        env.set("quote", Expr::Proc(builtins::quote));

        // arithmetic operations
        env.set("+", Expr::Proc(builtins::num::add));
        env.set("-", Expr::Proc(builtins::num::minus));
        env.set("*", Expr::Proc(builtins::num::multiply));
        env.set("/", Expr::Proc(builtins::num::divide));

        env
    }

    pub fn get(&self, name: &str) -> Option<&Expr> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: &str, expr: Expr) {
        let _ = self.vars.insert(name.to_string(), expr);
    }
}
