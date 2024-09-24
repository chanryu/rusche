use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::builtin::{self, load_builtin};
use crate::env::Env;
use crate::expr::Expr;
use crate::list::List;

pub type EvalError = String;
pub type EvalResult = Result<Expr, EvalError>;

#[derive(Clone, Debug)]
pub struct EvalContext {
    pub env: Rc<Env>,
}

impl EvalContext {
    pub fn derive_from(base: &EvalContext) -> Self {
        Self {
            env: Env::derive_from(&base.env),
        }
    }
}

pub fn eval(expr: &Expr, context: &EvalContext) -> EvalResult {
    use builtin::quote::{quasiquote, quote};

    match expr {
        Expr::Sym(name, _) => match context.env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(format!("Undefined symbol: {:?}", name)),
        },
        Expr::List(List::Cons(cons), _) => match cons.car.as_ref() {
            Expr::Sym(text, _) if text == "quote" => quote(text, &cons.cdr, context),
            Expr::Sym(text, _) if text == "quasiquote" => quasiquote(text, &cons.cdr, context),
            _ => {
                if let Expr::Proc(proc, _) = eval(&cons.car, context)? {
                    let args = &cons.cdr;
                    proc.invoke(args, context)
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
    context: EvalContext,
}

impl Evaluator {
    pub fn new() -> Self {
        let all_envs = Rc::new(RefCell::new(Vec::new()));
        let root_env = Env::root(Rc::downgrade(&all_envs));

        all_envs.borrow_mut().push(Rc::downgrade(&root_env));

        Self {
            all_envs,
            context: EvalContext { env: root_env },
        }
    }

    pub fn with_builtin() -> Self {
        let evaluator = Self::new();
        load_builtin(&evaluator.root_env());
        evaluator
    }

    pub fn root_env(&self) -> &Rc<Env> {
        return &self.context.env;
    }

    pub fn context(&self) -> &EvalContext {
        &self.context
    }

    pub fn eval(&self, expr: &Expr) -> EvalResult {
        let result = eval(expr, &self.context());

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
