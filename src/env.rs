use crate::built_in;
use crate::expr::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Env {
    base: Option<Box<Env>>,
    vars: Rc<RefCell<HashMap<String, Expr>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            base: None,
            vars: Rc::new(RefCell::new(HashMap::new())),
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
        env.set("lambda", Expr::new_native_proc(built_in::lambda));
        env.set("quote", Expr::new_native_proc(built_in::quote));

        // arithmetic operations
        env.set("+", Expr::new_native_proc(built_in::num::add));
        env.set("-", Expr::new_native_proc(built_in::num::minus));
        env.set("*", Expr::new_native_proc(built_in::num::multiply));
        env.set("/", Expr::new_native_proc(built_in::num::divide));

        env
    }

    pub fn lookup(&self, name: &str) -> Option<Expr> {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow().get(name) {
                return Some(value.clone());
            }
            if let Some(base) = &env.base {
                env = base;
            } else {
                break;
            }
        }
        None
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.vars.borrow_mut().insert(name.into(), expr);
    }

    pub fn derive(&self) -> Env {
        let mut derived_env = Env::new();
        derived_env.base = Some(Box::new(self.clone()));
        derived_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;

    #[test]
    fn test_lookup() {
        let env = Env::new();
        assert_eq!(env.lookup("one"), None);
        env.set("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive() {
        let base = Env::new();
        let derived = base.derive();

        assert_eq!(derived.lookup("two"), None);
        base.set("two", num(2));
        assert_eq!(derived.lookup("two"), Some(num(2)));

        derived.set("three", num(3));
        assert_eq!(base.lookup("three"), None);
        assert_eq!(derived.lookup("three"), Some(num(3)));
    }

    #[test]
    fn test_clone() {
        let original = Env::new();
        let cloned = original.clone();

        original.set("one", num(1));
        assert_eq!(cloned.lookup("one"), Some(num(1)));
    }
}
