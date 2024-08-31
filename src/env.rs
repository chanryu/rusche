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

        let set_native_func = |name, func| {
            env.set(name, Expr::new_native_proc(func));
        };

        // lisp built_in
        set_native_func("atom", built_in::atom);
        set_native_func("atom", built_in::atom);
        set_native_func("car", built_in::car);
        set_native_func("cdr", built_in::cdr);
        set_native_func("cond", built_in::cond);
        set_native_func("define", built_in::define);
        set_native_func("eq", built_in::eq);
        set_native_func("lambda", built_in::lambda);

        // quote
        set_native_func("quote", built_in::quote::quote);
        set_native_func("quasiquote", built_in::quote::quasiquote);
        set_native_func("unquote", built_in::quote::unquote);
        set_native_func("unquote-splicing", built_in::quote::unquote_splicing);

        // arithmetic operations
        set_native_func("+", built_in::num::add);
        set_native_func("-", built_in::num::minus);
        set_native_func("*", built_in::num::multiply);
        set_native_func("/", built_in::num::divide);

        env
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.vars.borrow_mut().insert(name.into(), expr);
    }

    pub fn update(&self, name: &str, expr: Expr) -> bool {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow_mut().get_mut(name) {
                *value = expr;
                return true;
            }
            if let Some(base) = &env.base {
                env = base;
            } else {
                return false;
            }
        }
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

    pub fn derive(&self) -> Env {
        let mut derived_env = Env::new();
        derived_env.base = Some(Box::new(self.clone()));
        derived_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::{num, str};

    #[test]
    fn test_set() {
        let env = Env::new();
        assert_eq!(env.vars.borrow().len(), 0);
        env.set("one", num(1));
        assert_eq!(env.vars.borrow().get("one"), Some(&num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::new();
        assert_eq!(env.update("name", num(1)), false);

        env.set("name", num(0));
        assert_eq!(env.update("name", num(1)), true);
    }

    #[test]
    fn test_lookup() {
        let env = Env::new();
        assert_eq!(env.lookup("one"), None);
        env.set("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive_update() {
        let base = Env::new();
        let derived = base.derive();

        base.set("one", num(1));
        derived.set("two", num(2));

        assert_eq!(derived.update("one", str("uno")), true);
        assert_eq!(derived.update("two", str("dos")), true);

        assert_eq!(base.vars.borrow().get("one"), Some(&str("uno")));
        assert_eq!(derived.vars.borrow().get("one"), None);
        assert_eq!(derived.vars.borrow().get("two"), Some(&str("dos")));
    }

    #[test]
    fn test_derive_lookup() {
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
