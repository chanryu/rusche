use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::eval::ForeignTracers;
use crate::expr::Expr;
use crate::proc::{NativeFunc, Proc};
use crate::symbol::Symbol;

/// Storage for bindings. The root environment uses a hash map (many builtins); derived
/// frames use a small vector because they typically hold only a few locals.
#[derive(Debug)]
enum Vars {
    Map(HashMap<Symbol, Expr>),
    Flat(Vec<(Symbol, Expr)>),
}

impl Vars {
    fn define(&mut self, name: Symbol, expr: Expr) {
        match self {
            Vars::Map(map) => {
                map.insert(name, expr);
            }
            Vars::Flat(flat) => {
                if let Some((_, slot)) = flat.iter_mut().find(|(k, _)| k == &name) {
                    *slot = expr;
                } else {
                    flat.push((name, expr));
                }
            }
        }
    }

    fn update(&mut self, name: &Symbol, expr: Expr) -> bool {
        match self {
            Vars::Map(map) => {
                if let Some(slot) = map.get_mut(name) {
                    *slot = expr;
                    true
                } else {
                    false
                }
            }
            Vars::Flat(flat) => {
                if let Some((_, slot)) = flat.iter_mut().find(|(k, _)| k == name) {
                    *slot = expr;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn lookup(&self, name: &Symbol) -> Option<Expr> {
        match self {
            Vars::Map(map) => map.get(name).cloned(),
            Vars::Flat(flat) => flat
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone()),
        }
    }

    fn names(&self) -> Vec<String> {
        match self {
            Vars::Map(map) => map.keys().map(|k| k.to_string()).collect(),
            Vars::Flat(flat) => flat.iter().map(|(k, _)| k.to_string()).collect(),
        }
    }

    fn values(&self) -> Vec<Expr> {
        match self {
            Vars::Map(map) => map.values().cloned().collect(),
            Vars::Flat(flat) => flat.iter().map(|(_, v)| v.clone()).collect(),
        }
    }

    fn clear(&mut self) {
        match self {
            Vars::Map(map) => map.clear(),
            Vars::Flat(flat) => flat.clear(),
        }
    }

    #[cfg(test)]
    fn get(&self, name: &str) -> Option<Expr> {
        self.lookup(&Symbol::intern(name))
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        match self {
            Vars::Map(map) => map.len(),
            Vars::Flat(flat) => flat.len(),
        }
    }
}

/// `Env` object stores variable bindings and manages scope for expression evaluation.
///
/// The `Env` struct is used to create an environment for evaluating expressions.
/// It supports nested scopes by maintaining a reference to a base environment.
/// It also keeps track of all environments and their reachability status.
#[derive(Debug)]
pub struct Env {
    base: Option<Rc<Env>>,
    vars: RefCell<Vars>,
    all_envs: Weak<RefCell<Vec<Weak<Env>>>>,
    is_reachable: Cell<bool>,
    /// Whether this env is in the GC registry. Derived frames register lazily when a
    /// closure or macro captures them -- most call frames never need it.
    registered: Cell<bool>,
}

impl Env {
    pub(crate) fn root(all_envs: Weak<RefCell<Vec<Weak<Env>>>>) -> Rc<Self> {
        Rc::new(Self {
            base: None,
            vars: RefCell::new(Vars::Map(HashMap::new())),
            all_envs,
            is_reachable: Cell::new(false),
            registered: Cell::new(true), // root is always registered by Evaluator::new
        })
    }

    pub(crate) fn derive_from(base: &Rc<Env>) -> Rc<Self> {
        Rc::new(Self {
            base: Some(base.clone()),
            vars: RefCell::new(Vars::Flat(Vec::new())),
            all_envs: base.all_envs.clone(),
            is_reachable: Cell::new(false),
            registered: Cell::new(false),
        })
    }

    /// Registers this environment with the garbage collector if it is not already.
    ///
    /// Call this when a closure or macro captures the environment, forming a potential
    /// reference cycle that ordinary `Rc` dropping cannot break.
    pub(crate) fn ensure_registered(self: &Rc<Self>) {
        if self.registered.get() {
            return;
        }
        let Some(all_envs) = self.all_envs.upgrade() else {
            return;
        };
        let mut all_envs = all_envs.borrow_mut();
        // Drop registry entries for environments that reference counting has already
        // freed. Doing this only when the vector is full keeps the cost amortised O(1)
        // while preventing unbounded growth in long-running loops.
        if all_envs.len() == all_envs.capacity() {
            all_envs.retain(|env| env.strong_count() > 0);
        }
        all_envs.push(Rc::downgrade(self));
        self.registered.set(true);
    }

    /// Defines a new variable binding in the current environment.
    ///
    /// This function inserts a new variable binding into the current environment's variable map.
    /// If the variable already exists, its binding will be overwritten with the new expression.
    ///
    /// # Type Parameters
    ///
    /// * `IntoExpr` - A type that can be converted into an `Expr`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to define.
    /// * `expr` - The expression to bind to the variable. This can be any type that implements the `Into<Expr>` trait.
    pub fn define<IntoExpr>(&self, name: impl AsRef<str>, expr: IntoExpr)
    where
        IntoExpr: Into<Expr>,
    {
        self.define_sym(Symbol::intern(name), expr.into());
    }

    /// Defines a binding using an already-interned [`Symbol`].
    pub fn define_sym(&self, name: Symbol, expr: impl Into<Expr>) {
        self.vars.borrow_mut().define(name, expr.into());
    }

    /// Updates a variable binding in the environment.
    ///
    /// This function first searches for the variable in the current environment.
    /// If the variable is found, it updates the binding with the new expression.
    /// If the variable is not found, it recursively searches in the base environment.
    /// If the variable is not found in any ancestor environments, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to update.
    /// * `expr` - The expression to bind to the variable.
    ///
    /// # Returns
    ///
    /// Returns `true` if the variable was successfully updated, `false` otherwise.
    pub fn update<IntoExpr>(&self, name: impl AsRef<str>, expr: IntoExpr) -> bool
    where
        IntoExpr: Into<Expr>,
    {
        self.update_sym(&Symbol::intern(name), expr.into())
    }

    /// Updates a binding using an already-interned [`Symbol`].
    pub fn update_sym(&self, name: &Symbol, expr: impl Into<Expr>) -> bool {
        let expr = expr.into();
        let mut env = self;
        loop {
            if env.vars.borrow_mut().update(name, expr.clone()) {
                return true;
            }
            let Some(base) = &env.base else {
                return false;
            };
            env = base;
        }
    }

    /// Looks up the binding for the given name.
    ///
    /// This function first searches for the binding in the current environment.
    /// If the binding is not found, it recursively searches in all ancestor environments.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the variable to look up.
    ///
    /// # Returns
    ///
    /// Returns an `Option` containing the expression bound to the variable if found, or `None` if not found.
    pub fn lookup(&self, name: impl AsRef<str>) -> Option<Expr> {
        self.lookup_sym(&Symbol::intern(name))
    }

    /// Looks up a binding using an already-interned [`Symbol`].
    pub fn lookup_sym(&self, name: &Symbol) -> Option<Expr> {
        let mut env = self;
        loop {
            if let Some(value) = env.vars.borrow().lookup(name) {
                return Some(value);
            }
            let Some(base) = &env.base else {
                return None;
            };
            env = base;
        }
    }

    /// Returns the names bound in this environment and all ancestor environments.
    ///
    /// Names from ancestor environments appear first; names defined in this environment
    /// appear last. Duplicates are possible if a name is shadowed.
    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut env = self;
        loop {
            names.extend(env.vars.borrow().names());
            let Some(base) = &env.base else {
                break;
            };
            env = base;
        }
        names
    }

    /// A convience fucntion to define a native procedure in the current environment.
    /// This is a shorthand for `define(name, Expr::Proc(Proc::Native { ... }))`.
    pub fn define_native_proc(&self, name: &str, func: NativeFunc) {
        self.define(
            name,
            Expr::Proc(
                Rc::new(Proc::Native {
                    name: Rc::from(name),
                    func,
                }),
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

    pub(crate) fn gc_mark(&self, tracers: &ForeignTracers) {
        if self.is_reachable.get() {
            return;
        }

        self.is_reachable.set(true);

        for expr in self.vars.borrow().values() {
            Self::gc_mark_expr(&expr, tracers);
        }
    }

    /// Marks every environment reachable from `expr`: closures captured directly, inside
    /// lists, or inside foreign objects with a registered tracer.
    pub(crate) fn gc_mark_expr(expr: &Expr, tracers: &ForeignTracers) {
        match expr {
            Expr::Proc(proc, _) => Self::gc_mark_proc(proc.as_ref(), tracers),
            Expr::List(list, _) => list
                .iter()
                .for_each(|expr| Self::gc_mark_expr(expr, tracers)),
            Expr::Foreign(object) => {
                if let Some(tracer) = tracers.get(&object.as_ref().type_id()) {
                    tracer(object.as_ref(), &mut |expr| {
                        Self::gc_mark_expr(expr, tracers)
                    });
                }
            }
            Expr::Num(..) | Expr::Bool(..) | Expr::Str(..) | Expr::Sym(..) => {}
        }
    }

    fn gc_mark_proc(proc: &Proc, tracers: &ForeignTracers) {
        if let Proc::Closure { outer_context, .. } = proc {
            outer_context.env.gc_mark(tracers);
        }
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
        assert_eq!(env.vars.borrow().get("one"), Some(num(1)));
    }

    #[test]
    fn test_update() {
        let env = Env::root(Weak::new());
        assert!(!env.update("name", 1));

        env.define("name", 0);
        assert!(env.update("name", 1));
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

        assert!(derived.update("one", "uno"));
        assert!(derived.update("two", "dos"));

        assert_eq!(base.vars.borrow().get("one"), Some("uno".into()));
        assert_eq!(derived.vars.borrow().get("one"), None);
        assert_eq!(derived.vars.borrow().get("two"), Some("dos".into()));
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

    #[test]
    fn test_names_walks_ancestors() {
        let base = Env::root(Weak::new());
        base.define("a", 1);
        let derived = Env::derive_from(&base);
        derived.define("b", 2);

        let names = derived.names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }
}
