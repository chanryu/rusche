use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::expr::Expr;
use crate::proc::{NativeFunc, Proc};

#[derive(Debug)]
pub struct Env {
    base: Option<Rc<Env>>,
    vars: RefCell<HashMap<String, Expr>>,
    all_envs: Weak<RefCell<Vec<Weak<Env>>>>,
    is_reachable: Cell<bool>,
}

impl Env {
    pub(crate) fn root(all_envs: Weak<RefCell<Vec<Weak<Env>>>>) -> Rc<Self> {
        Rc::new(Self {
            base: None,
            vars: RefCell::new(HashMap::new()),
            all_envs,
            is_reachable: Cell::new(false),
        })
    }

    pub(crate) fn derive_from(base: &Rc<Env>) -> Rc<Self> {
        let derived_env = Rc::new(Self {
            base: Some(base.clone()),
            vars: RefCell::new(HashMap::new()),
            all_envs: base.all_envs.clone(),
            is_reachable: Cell::new(false),
        });

        if let Some(all_envs) = base.all_envs.upgrade() {
            all_envs.borrow_mut().push(Rc::downgrade(&derived_env));
        }

        derived_env
    }

    pub fn define<IntoExpr>(&self, name: &str, expr: IntoExpr)
    where
        IntoExpr: Into<Expr>,
    {
        self.vars.borrow_mut().insert(name.into(), expr.into());
    }

    pub fn update<IntoExpr>(&self, name: &str, expr: IntoExpr) -> bool
    where
        IntoExpr: Into<Expr>,
    {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow_mut().get_mut(name) {
                *value = expr.into();
                return true;
            }
            let Some(base) = &env.base else {
                return false;
            };
            env = base;
        }
    }

    pub fn lookup(&self, name: &str) -> Option<Expr> {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow().get(name) {
                return Some(value.clone());
            }
            let Some(base) = &env.base else {
                return None;
            };
            env = base;
        }
    }

    pub fn define_native_proc(&self, name: &str, func: NativeFunc) {
        self.define(
            name,
            Expr::Proc(
                Proc::Native {
                    name: name.to_owned(),
                    func,
                },
                None,
            ),
        );
    }
}

/// Garbage collection
impl Env {
    pub(crate) fn gc_prepare(&self) {
        self.is_reachable.set(false);
    }

    pub(crate) fn gc_mark(&self) {
        if self.is_reachable.get() {
            return;
        }

        self.is_reachable.set(true);

        self.vars.borrow().values().for_each(|expr| {
            if let Expr::Proc(Proc::Closure { outer_context, .. }, _) = expr {
                outer_context.env.gc_mark();
            }
        });
    }

    pub(crate) fn gc_sweep(&self) {
        self.vars.borrow_mut().clear();
    }

    pub(crate) fn is_reachable(&self) -> bool {
        self.is_reachable.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::num;

    #[test]
    fn test_set() {
        let env = Env::root(Weak::new());
        assert_eq!(env.vars.borrow().len(), 0);
        env.define("one", 1);
        assert_eq!(env.vars.borrow().get("one"), Some(&num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::root(Weak::new());
        assert_eq!(env.update("name", 1), false);

        env.define("name", 0);
        assert_eq!(env.update("name", 1), true);
    }

    #[test]
    fn test_lookup() {
        let env = Env::root(Weak::new());
        assert_eq!(env.lookup("one"), None);
        env.define("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive_update() {
        let base = Env::root(Weak::new());
        let derived = Env::derive_from(&base);

        base.define("one", 1);
        derived.define("two", 2);

        assert_eq!(derived.update("one", "uno"), true);
        assert_eq!(derived.update("two", "dos"), true);

        assert_eq!(base.vars.borrow().get("one"), Some(&"uno".into()));
        assert_eq!(derived.vars.borrow().get("one"), None);
        assert_eq!(derived.vars.borrow().get("two"), Some(&"dos".into()));
    }

    #[test]
    fn test_derive_lookup() {
        let base = Env::root(Weak::new());
        let derived = Env::derive_from(&base);

        assert_eq!(derived.lookup("two"), None);
        base.define("two", 2);
        assert_eq!(derived.lookup("two"), Some(num(2)));

        derived.define("three", 3);
        assert_eq!(base.lookup("three"), None);
        assert_eq!(derived.lookup("three"), Some(num(3)));
    }

    #[test]
    fn test_clone() {
        let original = Env::root(Weak::new());
        let cloned = original.clone();

        original.define("one", 1);
        assert_eq!(cloned.lookup("one"), Some(num(1)));
    }
}
