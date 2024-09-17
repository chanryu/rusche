use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::env::Env;
use crate::expr::Expr;
use crate::list::List;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Rc<Env>) -> EvalResult {
    match expr {
        Expr::Sym(name) => match env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(List::Cons(cons)) => {
            if let Expr::Proc(proc) = eval(&cons.car, env)? {
                let args = &cons.cdr;
                proc.invoke(args, env)
            } else {
                Err(format!("{} does not evaluate to a callable.", cons.car))
            }
        }
        _ => Ok(expr.clone()),
    }
}

pub struct EvalContext {
    all_envs: Rc<RefCell<Vec<Weak<Env>>>>,

    #[cfg(not(debug_assertions))]
    root_env: Rc<Env>,

    #[cfg(debug_assertions)]
    root_env: Option<Rc<Env>>,
}

impl EvalContext {
    pub fn new() -> Self {
        let all_envs = Rc::new(RefCell::new(Vec::new()));
        let root_env = Env::root(all_envs.clone());

        all_envs.borrow_mut().push(Rc::downgrade(&root_env));

        Self {
            all_envs,

            #[cfg(not(debug_assertions))]
            root_env,

            #[cfg(debug_assertions)]
            root_env: Some(root_env),
        }
    }

    pub fn root_env(&self) -> &Rc<Env> {
        #[cfg(not(debug_assertions))]
        return &self.root_env;

        #[cfg(debug_assertions)]
        self.root_env
            .as_ref()
            .unwrap_or_else(|| panic!("Root environment is unavailable."))
    }

    pub fn eval(&self, expr: &Expr) -> EvalResult {
        let result = eval(expr, self.root_env());

        // TODO: Collect garbage if needed

        result
    }

    pub fn collect_garbage(&self) {
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.is_reachable.set(false);
            }
        });

        self.root_env().gc_mark_reachable();

        #[cfg(debug_assertions)]
        println!(
            "GC: Unreachable envs: {}",
            self.all_envs
                .borrow()
                .iter()
                .filter(|env| {
                    let Some(env) = env.upgrade() else {
                        return false;
                    };
                    !env.is_reachable.get()
                })
                .count()
        );

        // GC sweep
        let reachable_envs = self
            .all_envs
            .borrow()
            .iter()
            .filter(|env| {
                let Some(env) = env.upgrade() else {
                    return false;
                };
                if !env.is_reachable.get() {
                    env.clear();
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        *self.all_envs.borrow_mut() = reachable_envs;
    }
}

impl Drop for EvalContext {
    fn drop(&mut self) {
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.clear();
            }
        });

        #[cfg(debug_assertions)]
        {
            self.root_env = None;
            debug_assert_eq!(
                0,
                self.all_envs
                    .borrow()
                    .iter()
                    .filter(|env| env.upgrade().is_some())
                    .count()
            );
        }
    }
}
