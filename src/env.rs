use crate::built_in;
use crate::expr::Expr;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
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

        // lisp built_in
        env.set("atom", Expr::new_native_proc(built_in::atom));
        env.set("car", Expr::new_native_proc(built_in::car));
        env.set("cdr", Expr::new_native_proc(built_in::cdr));
        env.set("cond", Expr::new_native_proc(built_in::cond));
        env.set("define", Expr::new_native_proc(built_in::define));
        env.set("eq", Expr::new_native_proc(built_in::eq));
        env.set("quote", Expr::new_native_proc(built_in::quote));

        // arithmetic operations
        env.set("+", Expr::new_native_proc(built_in::num::add));
        env.set("-", Expr::new_native_proc(built_in::num::minus));
        env.set("*", Expr::new_native_proc(built_in::num::multiply));
        env.set("/", Expr::new_native_proc(built_in::num::divide));

        env
    }

    pub fn get(&self, name: &str) -> Option<Expr> {
        self.vars.borrow().get(name).cloned()
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.vars.borrow_mut().insert(name.into(), expr);
    }
}
