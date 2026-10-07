use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
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

/// The default maximum number of nested procedure calls. See [`Evaluator::set_max_call_depth`].
///
/// This value is chosen to stay within the 8 MiB main-thread stack of a debug build. If you
/// evaluate on a thread with a smaller stack, lower it; on a release build with a large stack,
/// you can raise it.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 1000;

/// The default number of live environments that triggers automatic garbage collection.
/// See [`Evaluator::set_gc_threshold`].
pub const DEFAULT_GC_THRESHOLD: usize = 10_000;

/// A callback that reports every [`Expr`] held inside a [`Foreign`](crate::expr::Foreign)
/// object so the garbage collector can keep the environments they reference alive.
/// Register one per concrete type with [`Evaluator::register_foreign_tracer`].
pub type ForeignTracer = Box<dyn Fn(&dyn Any, &mut dyn FnMut(&Expr))>;

pub(crate) type ForeignTracers = HashMap<TypeId, ForeignTracer>;

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
    max_call_depth: Rc<Cell<usize>>,

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
            max_call_depth: base.max_call_depth.clone(),
            #[cfg(feature = "callstack_trace")]
            call_stack: base.call_stack.clone(),
        }
    }

    pub(crate) fn push_call(&self, proc: &Proc) -> Result<(), EvalError> {
        #[cfg(not(feature = "callstack_trace"))]
        let _ = proc;

        let depth = self.call_depth.get();
        let max_depth = self.max_call_depth.get();
        if depth >= max_depth {
            return Err(EvalError::from(format!(
                "Maximum call depth ({max_depth}) exceeded."
            )));
        }
        self.call_depth.set(depth + 1);

        #[cfg(feature = "callstack_trace")]
        {
            self.call_stack.borrow_mut().push(proc.badge());
            println!("{:03}{} -> {}", depth, " ".repeat(depth), proc.badge());
        }

        Ok(())
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

            let result = match &cons.car {
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
                    let span = if let Some(span) = cons.cdr.span() {
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
                args: args.clone(),
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
///
/// # Garbage collection
///
/// Environments form reference cycles through closures, so they cannot be freed by reference
/// counting alone. The evaluator keeps a registry of every environment and collects the
/// unreachable ones:
///
/// - automatically, after a top-level [`Evaluator::eval`] returns, once the number of registered
///   environments reaches a threshold (see [`Evaluator::set_gc_threshold`]), or
/// - manually, via [`Evaluator::collect_garbage`].
///
/// Reachability starts from the root environment and the value just returned by `eval`.
/// Anything the host keeps alive outside of the evaluator (for example an `Expr::Proc` stored in
/// a Rust struct) is **not** a root; define it in the root environment or keep it inside a
/// [`Foreign`](crate::expr::Foreign) object with a registered
/// [tracer](Evaluator::register_foreign_tracer) if it must survive collection.
pub struct Evaluator {
    all_envs: Rc<RefCell<Vec<Weak<Env>>>>,
    context: EvalContext,
    foreign_tracers: RefCell<ForeignTracers>,
    gc_threshold: Cell<Option<usize>>,
    next_gc_at: Cell<usize>,
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
                max_call_depth: Rc::new(Cell::new(DEFAULT_MAX_CALL_DEPTH)),
                #[cfg(feature = "callstack_trace")]
                call_stack: Rc::new(RefCell::new(Vec::new())),
            },
            foreign_tracers: RefCell::new(HashMap::new()),
            gc_threshold: Cell::new(Some(DEFAULT_GC_THRESHOLD)),
            next_gc_at: Cell::new(DEFAULT_GC_THRESHOLD),
        }
    }

    /// Returns the maximum number of nested procedure calls allowed before evaluation fails
    /// with an error instead of overflowing the Rust stack.
    pub fn max_call_depth(&self) -> usize {
        self.context.max_call_depth.get()
    }

    /// Sets the maximum number of nested procedure calls. Defaults to
    /// [`DEFAULT_MAX_CALL_DEPTH`].
    ///
    /// Every procedure invocation (native, closure, or macro) counts as one level while it is
    /// active; tail calls do not accumulate. Pick a value that fits the stack of the thread you
    /// evaluate on -- roughly 6 KiB per level in debug builds and 2 KiB in release builds.
    pub fn set_max_call_depth(&self, depth: usize) {
        self.context.max_call_depth.set(depth);
    }

    /// Returns the garbage collection threshold, or `None` if automatic collection is disabled.
    pub fn gc_threshold(&self) -> Option<usize> {
        self.gc_threshold.get()
    }

    /// Sets the number of registered environments that triggers an automatic garbage collection
    /// after a top-level [`Evaluator::eval`]. Pass `None` to disable automatic collection and
    /// rely on [`Evaluator::collect_garbage`] instead. Defaults to [`DEFAULT_GC_THRESHOLD`].
    pub fn set_gc_threshold(&self, threshold: Option<usize>) {
        self.gc_threshold.set(threshold);
        if let Some(threshold) = threshold {
            self.next_gc_at.set(threshold);
        }
    }

    /// Registers a tracer for [`Foreign`](crate::expr::Foreign) objects of type `T`.
    ///
    /// When the garbage collector encounters an `Expr::Foreign` whose payload is a `T`, it calls
    /// `tracer` and treats every `Expr` passed to the callback as reachable. Without a tracer,
    /// closures stored inside a foreign object may have their environments collected.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{cell::RefCell, rc::Rc};
    /// use rusche::{Evaluator, Expr};
    ///
    /// type ExprVec = RefCell<Vec<Expr>>;
    ///
    /// let evaluator = Evaluator::default();
    /// evaluator.register_foreign_tracer::<ExprVec>(|vec, trace| {
    ///     vec.borrow().iter().for_each(trace);
    /// });
    /// ```
    pub fn register_foreign_tracer<T: Any>(
        &self,
        tracer: impl Fn(&T, &mut dyn FnMut(&Expr)) + 'static,
    ) {
        self.foreign_tracers.borrow_mut().insert(
            TypeId::of::<T>(),
            Box::new(move |object, trace| {
                if let Some(object) = object.downcast_ref::<T>() {
                    tracer(object, trace);
                }
            }),
        );
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
    ///
    /// After the evaluation finishes, garbage is collected automatically if the number of
    /// registered environments has reached the [threshold](Evaluator::set_gc_threshold).
    /// The returned value is treated as a root during that collection.
    pub fn eval(&self, expr: &Expr) -> EvalResult {
        let result = eval(expr, self.context());
        self.maybe_collect_garbage(result.as_ref().ok());
        result
    }

    /// Count the number of unreachable environments in the evaluator.
    /// This function is useful for monitoring memory usage and can be used
    /// to determin when to trigger garbage collection.
    pub fn count_unreachable_envs(&self) -> usize {
        self.gc_mark(&[]);

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
    ///
    /// Every environment that is not reachable from the root environment is emptied, which
    /// breaks the reference cycles between environments and the closures they hold.
    /// See the [type-level documentation](Evaluator#garbage-collection) for what counts as
    /// reachable.
    pub fn collect_garbage(&self) {
        self.collect_garbage_with_roots(&[]);
    }

    fn maybe_collect_garbage(&self, extra_root: Option<&Expr>) {
        let Some(threshold) = self.gc_threshold.get() else {
            return;
        };
        if self.context.is_in_proc() || self.all_envs.borrow().len() < self.next_gc_at.get() {
            return;
        }

        self.collect_garbage_with_roots(extra_root.as_slice());

        // Back off proportionally to the survivors so a large live heap does not trigger a
        // full collection after every evaluation.
        let survivors = self.all_envs.borrow().len();
        self.next_gc_at.set(threshold.max(survivors * 2));
    }

    fn gc_mark(&self, extra_roots: &[&Expr]) {
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.gc_prepare();
            }
        });

        let tracers = self.foreign_tracers.borrow();
        self.root_env().gc_mark(&tracers);
        for expr in extra_roots {
            Env::gc_mark_expr(expr, &tracers);
        }
    }

    fn collect_garbage_with_roots(&self, extra_roots: &[&Expr]) {
        self.gc_mark(extra_roots);

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
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        *self.all_envs.borrow_mut() = reachable_envs;
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::with_prelude()
    }
}

impl Drop for Evaluator {
    fn drop(&mut self) {
        // Break every env <-> closure cycle so the environments can be freed. Expressions the
        // host still holds (e.g. a closure returned from `eval`) legitimately keep their
        // environment allocation alive beyond this point; that is fine, it is just empty now.
        self.all_envs.borrow().iter().for_each(|env| {
            if let Some(env) = env.upgrade() {
                env.gc_sweep()
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::tokenize, parser::Parser};

    /// Evaluates every top-level form in `src` and returns the last result.
    fn eval_all(evaluator: &Evaluator, src: &str) -> Expr {
        let mut parser = Parser::with_tokens(tokenize(src, None).unwrap());
        let mut last = Expr::from(List::Nil);
        while let Some(expr) = parser.parse().unwrap() {
            last = evaluator.eval(&expr).unwrap();
        }
        last
    }

    #[test]
    fn test_gc_threshold_accessors() {
        let evaluator = Evaluator::new();
        assert_eq!(evaluator.gc_threshold(), Some(DEFAULT_GC_THRESHOLD));

        evaluator.set_gc_threshold(Some(42));
        assert_eq!(evaluator.gc_threshold(), Some(42));

        evaluator.set_gc_threshold(None);
        assert_eq!(evaluator.gc_threshold(), None);
    }

    #[test]
    fn test_disabled_auto_gc_leaves_cycles_alone() {
        let evaluator = Evaluator::with_prelude();
        evaluator.set_gc_threshold(None);

        // Each `(make)` leaves behind an env <-> closure cycle (`get` lives in the env it
        // captures) that only a GC can reclaim.
        let src = "(define (make) (define n 0) (define (get) n) get)
                   (define i 0)
                   (while (< i 20) (make) (set! i (+ i 1)))";
        eval_all(&evaluator, src);

        assert!(evaluator.count_unreachable_envs() >= 20);
        evaluator.collect_garbage();
        assert_eq!(evaluator.count_unreachable_envs(), 0);
    }

    #[test]
    fn test_gc_marks_through_pending_tail_call() {
        let evaluator = Evaluator::with_prelude();

        let src = "(define (make-counter) (define n 0) (lambda () (set! n (+ n 1)) n))
                   (make-counter)";
        let counter_in_args = eval_all(&evaluator, src);
        let Expr::Proc(callee, _) = eval_all(&evaluator, "(make-counter)") else {
            panic!("expected a closure");
        };

        // A pending tail call references a procedure, its arguments, and the context it
        // will run in. All three must survive a collection.
        let context = EvalContext::derive_from(&evaluator.context);
        context.env.define("kept", 7);
        let tail_call = Expr::TailCall {
            proc: callee.clone(),
            args: List::from(vec![counter_in_args.clone()]),
            context: context.clone(),
        };
        evaluator.root_env().define("pending", tail_call);

        evaluator.collect_garbage();

        assert_eq!(context.env.lookup("kept"), Some(Expr::from(7)));
        evaluator
            .root_env()
            .define("callee", Expr::Proc(callee, None));
        evaluator.root_env().define("counter", counter_in_args);
        assert_eq!(
            evaluator.eval(&parse_one("(callee)")).unwrap(),
            Expr::from(1)
        );
        assert_eq!(
            evaluator.eval(&parse_one("(counter)")).unwrap(),
            Expr::from(1)
        );
    }

    #[test]
    fn test_gc_ignores_foreign_objects_without_tracer() {
        let evaluator = Evaluator::with_prelude();

        let closure = eval_all(
            &evaluator,
            "(define (make-counter) (define n 0) (lambda () (set! n (+ n 1)) n))
             (make-counter)",
        );
        let held: Rc<RefCell<Vec<Expr>>> = Rc::new(RefCell::new(vec![closure]));
        evaluator
            .root_env()
            .define("box", Expr::Foreign(held.clone()));

        // Without a registered tracer the closure inside the foreign object is not a root,
        // so its environment is swept and the closure no longer works.
        evaluator.collect_garbage();

        let closure = held.borrow()[0].clone();
        evaluator.root_env().define("counter", closure);
        assert!(evaluator.eval(&parse_one("(counter)")).is_err());
    }

    fn parse_one(src: &str) -> Expr {
        Parser::with_tokens(tokenize(src, None).unwrap())
            .parse()
            .unwrap()
            .unwrap()
    }
}
