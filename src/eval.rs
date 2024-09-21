use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::builtin::{self, load_builtin};
use crate::env::Env;
use crate::expr::{Expr, ExprKind};
use crate::list::List;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

pub fn eval(expr: &Expr, env: &Rc<Env>) -> EvalResult {
    use builtin::quote::{quasiquote, quote};

    match &expr.kind {
        ExprKind::Sym(name) => match env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        ExprKind::List(List::Cons(cons)) => match &cons.car.kind {
            ExprKind::Sym(text) if text == "quote" => quote(text, &cons.cdr, env),
            ExprKind::Sym(text) if text == "quasiquote" => quasiquote(text, &cons.cdr, env),
            _ => {
                if let ExprKind::Proc(proc) = eval(&cons.car, env)?.kind {
                    let args = &cons.cdr;
                    proc.invoke(args, env)
                } else {
                    Err(format!("{} does not evaluate to a callable.", cons.car))
                }
            }
        },
        _ => Ok(expr.clone()),
    }
}

pub struct Evaluator {
    all_envs: Rc<RefCell<Vec<Weak<Env>>>>,
    root_env: Rc<Env>,
}

impl Evaluator {
    pub fn new() -> Self {
        let all_envs = Rc::new(RefCell::new(Vec::new()));
        let root_env = Env::root(Rc::downgrade(&all_envs));

        all_envs.borrow_mut().push(Rc::downgrade(&root_env));

        Self { all_envs, root_env }
    }

    pub fn with_builtin() -> Self {
        let evaluator = Self::new();
        load_builtin(&evaluator.root_env());
        evaluator
    }

    pub fn root_env(&self) -> &Rc<Env> {
        return &self.root_env;
    }

    pub fn eval(&self, expr: &Expr) -> EvalResult {
        let result = eval(expr, self.root_env());

        // TODO: Collect garbage if needed

        result
    }

    pub fn collect_garbage(&self) {
        #[cfg(debug_assertions)]
        println!("GC: begin garbage collection");

        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.gc_prepare();
            }
        });

        self.root_env().gc_mark();

        #[cfg(debug_assertions)]
        let mut reachable_env_count = 0;

        // GC sweep
        let reachable_envs = self
            .all_envs
            .borrow()
            .iter()
            .filter(|env| {
                let Some(env) = env.upgrade() else {
                    return false;
                };
                if !env.is_reachable() {
                    env.gc_sweep();
                    #[cfg(debug_assertions)]
                    {
                        reachable_env_count += 1;
                    }
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        *self.all_envs.borrow_mut() = reachable_envs;

        #[cfg(debug_assertions)]
        println!(
            "GC: end garbage collection: {} envs recliamed",
            reachable_env_count
        );
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        self.all_envs.borrow().iter().for_each(|env| {
            env.upgrade().map(|env| env.gc_sweep());
        });

        // at this point, we should only have `root_env`
        debug_assert_eq!(
            1,
            self.all_envs
                .borrow()
                .iter()
                .filter(|env| env.upgrade().is_some())
                .count()
        );
    }
}
