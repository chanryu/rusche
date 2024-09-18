use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::builtin::load_builtin;
use crate::expr::Expr;
use crate::proc::Proc;

#[derive(Debug)]
enum EnvKind {
    Root {
        all_envs: Rc<RefCell<Vec<Weak<Env>>>>,
    },
    Derived {
        base: Rc<Env>,
    },
}

impl PartialEq for EnvKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Root { .. }, Self::Root { .. }) => true,
            (Self::Derived { base: base1 }, Self::Derived { base: base2 }) => {
                Rc::ptr_eq(base1, base2)
            }
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Env {
    kind: EnvKind,
    vars: RefCell<HashMap<String, Expr>>,
    pub(crate) is_reachable: Cell<bool>,
}

impl Env {
    pub(crate) fn root(all_envs: Rc<RefCell<Vec<Weak<Env>>>>) -> Rc<Self> {
        let env = Rc::new(Self {
            kind: EnvKind::Root { all_envs },
            vars: RefCell::new(HashMap::new()),
            is_reachable: Cell::new(false),
        });
        load_builtin(&env);
        env
    }

    pub fn derive_from(base: &Rc<Env>) -> Rc<Self> {
        let derived_env = Rc::new(Self {
            kind: EnvKind::Derived { base: base.clone() },
            vars: RefCell::new(HashMap::new()),
            is_reachable: Cell::new(false),
        });

        base.gc_register(&derived_env);

        derived_env
    }

    #[cfg(test)]
    pub(crate) fn for_unit_test() -> Rc<Self> {
        let env = Rc::new(Self {
            kind: EnvKind::Root {
                all_envs: Rc::new(RefCell::new(Vec::new())),
            },
            vars: RefCell::new(HashMap::new()),
            is_reachable: Cell::new(false),
        });

        env
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
            let EnvKind::Derived { base } = &env.kind else {
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
            let EnvKind::Derived { base } = &env.kind else {
                return None;
            };
            env = base;
        }
    }

    pub fn clear(&self) {
        self.vars.borrow_mut().clear();
    }

    pub(crate) fn gc_mark_reachable(&self) {
        if self.is_reachable.get() {
            return;
        }

        self.is_reachable.set(true);

        self.vars.borrow().values().for_each(|expr| {
            if let Expr::Proc(Proc::Closure { outer_env, .. }) = expr {
                outer_env.gc_mark_reachable();
            }
        });
    }

    fn gc_register(&self, derived_env: &Rc<Self>) {
        match &self.kind {
            EnvKind::Root { all_envs } => {
                all_envs.borrow_mut().push(Rc::downgrade(derived_env));
            }
            EnvKind::Derived { base } => {
                base.gc_register(derived_env);
            }
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for Env {
    fn drop(&mut self) {
        //decrement_env_count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::num;

    #[test]
    fn test_set() {
        let env = Env::for_unit_test();
        assert_eq!(env.vars.borrow().len(), 0);
        env.define("one", 1);
        assert_eq!(env.vars.borrow().get("one"), Some(&num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::for_unit_test();
        assert_eq!(env.update("name", 1), false);

        env.define("name", 0);
        assert_eq!(env.update("name", 1), true);
    }

    #[test]
    fn test_lookup() {
        let env = Env::for_unit_test();
        assert_eq!(env.lookup("one"), None);
        env.define("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive_update() {
        let base = Env::for_unit_test();
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
        let base = Env::for_unit_test();
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
        let original = Env::for_unit_test();
        let cloned = original.clone();

        original.define("one", 1);
        assert_eq!(cloned.lookup("one"), Some(num(1)));
    }
}
