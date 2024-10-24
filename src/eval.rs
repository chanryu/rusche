use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::{Rc, Weak},
};

use crate::{
    builtin::load_builtin,
    env::Env,
    expr::Expr,
    list::{Cons, List},
    prelude::load_prelude,
    proc::Proc,
    span::Span,
};

/// The object that represents an expression evaluation error.
#[derive(Debug, PartialEq)]
pub struct EvalError {
    pub message: String,
    pub span: Option<Span>,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(f, "{}: {}", span, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl From<String> for EvalError {
    fn from(message: String) -> Self {
        Self {
            message,
            span: None,
        }
    }
}

pub type EvalResult = Result<Expr, EvalError>;

/// The evaluation context contains the environment and other necessary state for expression evaluation.
#[derive(Clone, Debug)]
pub struct EvalContext {
    pub env: Rc<Env>,
    call_depth: Rc<Cell<usize>>,

    #[cfg(feature = "callstack_trace")]
    call_stack: Rc<RefCell<Vec<String>>>,
}

impl EvalContext {
    /// Derives a new evaluation context from the given base context.
    /// This function can be used to create a new context within a lambda or other procedure.
    pub fn derive_from(base: &EvalContext) -> Self {
        Self {
            env: Env::derive_from(&base.env),
            call_depth: base.call_depth.clone(),
            #[cfg(feature = "callstack_trace")]
            call_stack: base.call_stack.clone(),
        }
    }

    pub(crate) fn push_call(&self, proc: &Proc) {
        #[cfg(not(feature = "callstack_trace"))]
        let _ = proc;

        let depth = self.call_depth.get();
        self.call_depth.set(depth + 1);

        #[cfg(feature = "callstack_trace")]
        {
            self.call_stack.borrow_mut().push(proc.badge());
            println!("{:03}{} -> {}", depth, " ".repeat(depth), proc.badge());
        }
    }

    pub(crate) fn pop_call(&self) {
        self.call_depth.set(self.call_depth.get() - 1);

        #[cfg(feature = "callstack_trace")]
        {
            let badge = self.call_stack.borrow_mut().pop();
            if let Some(badge) = badge {
                let depth = self.call_depth.get();
                println!("{:03}{} <- {}", depth, " ".repeat(depth), badge);
            }
        }
    }

    pub(crate) fn is_in_proc(&self) -> bool {
        self.call_depth.get() > 0
    }
}

/// Evaluates an expression in the given context.
///
/// This function serves as the entry point for evaluating an expression.
/// It delegates the actual evaluation to the `eval_internal` function, specifying that the evaluation is not in a tail position.
///
/// # Arguments
///
/// * `expr` - A reference to the expression to be evaluated.
/// * `context` - A reference to the evaluation context, which includes the environment and other necessary state.
///
/// # Returns
///
/// Returns an `EvalResult`, which is typically a `Result` containing either the evaluated expression or an error.
pub fn eval(expr: &Expr, context: &EvalContext) -> EvalResult {
    eval_internal(expr, context, /*is_tail*/ false)
}

/// Evaluates an expression in the given context, denoting that the evaluation is in a tail position.
///
/// This function serves as the entry point for evaluating an expression with tail call optimization.
/// It delegates the actual evaluation to the `eval_internal` function, specifying that the evaluation is in a tail position.
///
/// # Arguments
///
/// * `expr` - A reference to the expression to be evaluated.
/// * `context` - A reference to the evaluation context, which includes the environment and other necessary state.
///
/// # Returns
///
/// Returns an `EvalResult`, which is typically a `Result` containing either the evaluated expression or an error.
pub fn eval_tail(expr: &Expr, context: &EvalContext) -> EvalResult {
    eval_internal(expr, context, /*is_tail*/ true)
}

fn eval_internal(expr: &Expr, context: &EvalContext, is_tail: bool) -> EvalResult {
    match expr {
        Expr::Sym(name, span) => match context.env.lookup(name) {
            Some(expr) => Ok(expr.clone()),
            None => Err(EvalError {
                message: format!("Undefined symbol: `{}`", name),
                span: *span,
            }),
        },
        Expr::List(List::Cons(cons), _) => {
            use crate::builtin::quote::{quasiquote, quote, QUASIQUOTE, QUOTE};

            let result = match cons.car.as_ref() {
                Expr::Sym(text, _) if text == QUOTE => quote(text, &cons.cdr, context),
                Expr::Sym(text, _) if text == QUASIQUOTE => quasiquote(text, &cons.cdr, context),
                _ => eval_s_expr(cons, context, is_tail),
            };

            match result {
                Err(EvalError {
                    message,
                    span: None,
                }) => {
                    // If the result is an error without a span, let's try to provide a span.
                    // First, let's check if we can get a span from arguments list. If not, we'll
                    // use the span of the expression itself.
                    let span = if let Some(span) = cons.cdr.as_ref().span() {
                        Some(span)
                    } else {
                        expr.span()
                    };
                    Err(EvalError { message, span })
                }
                _ => result,
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
            message: format!("`{}` does not evaluate to a callable.", s_expr.car),
            span: s_expr.car.span(),
        })
    }
}

/// The struct that encapsulates the evaluation environment, tail-call optimization context, and garbage collection.
/// It also maintains the evaluation context and provides utility functions to facilitate the evaluation process.
pub struct Evaluator {
    all_envs: Rc<RefCell<Vec<Weak<Env>>>>,
    context: EvalContext,
}

impl Evaluator {
    /// Creates a new `Evaluator` with the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - The evaluation context, which includes the environment and other necessary state.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Evaluator`.
    pub fn new() -> Self {
        let all_envs = Rc::new(RefCell::new(Vec::new()));
        let root_env = Env::root(Rc::downgrade(&all_envs));

        all_envs.borrow_mut().push(Rc::downgrade(&root_env));

        Self {
            all_envs,
            context: EvalContext {
                env: root_env,
                call_depth: Rc::new(Cell::new(0)),
                #[cfg(feature = "callstack_trace")]
                call_stack: Rc::new(RefCell::new(Vec::new())),
            },
        }
    }

    /// Creates a new `Evaluator` with built-in functions.
    pub fn with_builtin() -> Self {
        let evaluator = Self::new();
        load_builtin(evaluator.root_env());
        evaluator
    }

    /// Creates a new `Evaluator` with built-in functions and preludes.
    pub fn with_prelude() -> Self {
        let evaluator = Self::with_builtin();
        load_prelude(evaluator.context());
        evaluator
    }

    /// Returns the root environment of the evaluator.
    pub fn root_env(&self) -> &Rc<Env> {
        &self.context.env
    }

    /// Returns the evaluation context of the evaluator.
    pub fn context(&self) -> &EvalContext {
        &self.context
    }

    /// Evaluates an expression in the current context.
    /// This function is a convenience wrapper around the `eval()` function.
    pub fn eval(&self, expr: &Expr) -> EvalResult {
        eval(expr, self.context())
    }

    /// Count the number of unreachable environments in the evaluator.
    /// This function is useful for monitoring memory usage and can be used
    /// to determin when to trigger garbage collection.
    pub fn count_unreachable_envs(&self) -> usize {
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.gc_prepare();
            }
        });

        self.root_env().gc_mark();

        self.all_envs.borrow().iter().fold(0, |acc, env| {
            if let Some(env) = env.upgrade() {
                if !env.is_reachable() {
                    return acc + 1;
                }
            }
            acc
        })
    }

    /// Perform garbage collection on the evaluator.
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

impl Default for Evaluator {
    fn default() -> Self {
        Self::with_prelude()
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.gc_sweep()
            }
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
