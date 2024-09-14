use crate::expr::Expr;
use crate::prelude::load_prelude;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(debug_assertions)]
static mut GLOBAL_ENV_COUNTER: i32 = 0;

#[derive(Debug, PartialEq)]
pub struct Env {
    base: Option<Rc<Env>>,
    vars: RefCell<HashMap<String, Expr>>,
}

impl Env {
    pub(crate) fn new() -> Self {
        #[cfg(debug_assertions)]
        unsafe {
            GLOBAL_ENV_COUNTER += 1;
            println!("Env created: {}", GLOBAL_ENV_COUNTER);
        }
        Self {
            base: None,
            vars: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_prelude() -> Rc<Self> {
        let env = Rc::new(Self::new());
        load_prelude(&env);
        env
    }

    pub fn derive_from(base: &Rc<Env>) -> Rc<Self> {
        #[cfg(debug_assertions)]
        unsafe {
            GLOBAL_ENV_COUNTER += 1;
            println!("Env derived: {}", GLOBAL_ENV_COUNTER);
        }
        Rc::new(Self {
            base: Some(base.clone()),
            vars: RefCell::new(HashMap::new()),
        })
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
}

#[cfg(debug_assertions)]
impl Drop for Env {
    fn drop(&mut self) {
        unsafe {
            GLOBAL_ENV_COUNTER -= 1;
            println!("Env dropped: {}", GLOBAL_ENV_COUNTER);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::shortcuts::num;

    #[test]
    fn test_set() {
        let env = Env::new();
        assert_eq!(env.vars.borrow().len(), 0);
        env.define("one", 1);
        assert_eq!(env.vars.borrow().get("one"), Some(&num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::new();
        assert_eq!(env.update("name", 1), false);

        env.define("name", 0);
        assert_eq!(env.update("name", 1), true);
    }

    #[test]
    fn test_lookup() {
        let env = Env::new();
        assert_eq!(env.lookup("one"), None);
        env.define("one", num(1));
        assert_eq!(env.lookup("one"), Some(num(1)));
    }

    #[test]
    fn test_derive_update() {
        let base = Rc::new(Env::new());
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
        let base = Rc::new(Env::new());
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
        let original = Rc::new(Env::new());
        let cloned = original.clone();

        original.define("one", 1);
        assert_eq!(cloned.lookup("one"), Some(num(1)));
    }
}
