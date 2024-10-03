use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::builtin::load_builtin;
use crate::env::Env;
use crate::expr::Expr;
use crate::list::{Cons, List};
use crate::prelude::load_prelude;
use crate::proc::Proc;

pub use crate::eval_error::{EvalError, EvalErrorCode};
pub type EvalResult = Result<Expr, EvalError>;

#[cfg(debug_assertions)]
const TRACE_CALL_STACK: bool = false;

#[derive(Clone, Debug)]
pub struct EvalContext {
    pub env: Rc<Env>,
    call_depth: Rc<Cell<usize>>,

    #[cfg(debug_assertions)]
    call_stack: Rc<RefCell<Vec<String>>>,
}

impl EvalContext {
    pub fn derive_from(base: &EvalContext) -> Self {
        Self {
            env: Env::derive_from(&base.env),
            call_depth: base.call_depth.clone(),
            #[cfg(debug_assertions)]
            call_stack: base.call_stack.clone(),
        }
    }

    pub(crate) fn push_call(&self, proc: &Proc) {
        #[cfg(not(debug_assertions))]
        let _ = proc;

        let depth = self.call_depth.get();
        self.call_depth.set(depth + 1);

        #[cfg(debug_assertions)]
        {
            self.call_stack.borrow_mut().push(proc.identity());
            if TRACE_CALL_STACK {
                println!("{:03}{} -> {}", depth, " ".repeat(depth), proc.identity());
            }
        }
    }

    pub(crate) fn pop_call(&self) {
        self.call_depth.set(self.call_depth.get() - 1);

        #[cfg(debug_assertions)]
        {
            let identity = self.call_stack.borrow_mut().pop();

            if TRACE_CALL_STACK {
                if let Some(identity) = identity {
                    let depth = self.call_depth.get();
                    println!("{:03}{} <- {}", depth, " ".repeat(depth), identity);
                }
            }
        }
    }

    pub(crate) fn is_in_proc(&self) -> bool {
        self.call_depth.get() > 0
    }
}

pub fn eval(expr: &Expr, context: &EvalContext) -> EvalResult {
    eval_internal(expr, context, /*is_tail*/ false)
}

pub fn eval_tail(expr: &Expr, context: &EvalContext) -> EvalResult {
    eval_internal(expr, context, /*is_tail*/ true)
}

fn eval_internal(expr: &Expr, context: &EvalContext, is_tail: bool) -> EvalResult {
    match expr {
        Expr::Sym(name, span) => match context.env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(EvalError {
                code: EvalErrorCode::UndefinedSymbol,
                message: format!("Undefined symbol: `{}`", name),
                span: *span,
            }),
        },
        Expr::List(List::Cons(cons), _) => {
            use crate::builtin::quote::{quasiquote, quote};
            match cons.car.as_ref() {
                Expr::Sym(text, _) if text == "quote" => quote(text, &cons.cdr, context),
                Expr::Sym(text, _) if text == "quasiquote" => quasiquote(text, &cons.cdr, context),
                _ => {
                    let result = eval_s_expr(cons, context, is_tail);
                    match result {
                        Err(EvalError {
                            code,
                            message,
                            span: None,
                        }) => {
                            // TODO: pass proper span for code
                            Err(EvalError {
                                code,
                                message,
                                span: expr.span(),
                            })
                        }
                        _ => result,
                    }
                }
            }
        }
        _ => Ok(expr.clone()),
    }
}

fn eval_s_expr(s_expr: &Cons, context: &EvalContext, is_tail: bool) -> EvalResult {
    if let Expr::Proc(proc, _) = eval(&s_expr.car, context)? {
        let args = &s_expr.cdr;

        if is_tail && context.is_in_proc() {
            Ok(Expr::TailCall {
                proc: proc.clone(),
                args: args.as_ref().clone(),
                context: context.clone(),
            })
        } else {
            let mut res = proc.invoke(args, context)?;
            while let Expr::TailCall {
                proc,
                args,
                context,
            } = &res
            {
                res = proc.invoke(args, context)?;
            }
            Ok(res)
        }
    } else {
        Err(EvalError {
            code: EvalErrorCode::NotCallable,
            message: format!("{} does not evaluate to a callable.", s_expr.car),
            span: s_expr.car.span(),
        })
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
            context: EvalContext {
                env: root_env,
                call_depth: Rc::new(Cell::new(0)),
                #[cfg(debug_assertions)]
                call_stack: Rc::new(RefCell::new(Vec::new())),
            },
        }
    }

    pub fn with_builtin() -> Self {
        let evaluator = Self::new();
        load_builtin(&evaluator.root_env());
        evaluator
    }

    pub fn with_prelude() -> Self {
        let evaluator = Self::with_builtin();
        load_prelude(&evaluator.context());
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

        // at this point, we should only have `context.env`
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
