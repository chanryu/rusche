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
    error::Error,
    expr::{Expr, NIL},
    lexer::tokenize,
    list::{Cons, List},
    parser::Parser,
    prelude::load_prelude,
    proc::Proc,
    span::Span,
};

/// Tokenizes, parses, and evaluates every top-level expression in `src` in `context`.
///
/// Returns the value of the last expression, or [`NIL`] if `src` is empty.
/// When `strip_spans` is true, spans are removed from each parsed expression before
/// evaluation (used by the prelude so errors attribute to the call site).
pub fn eval_source(src: &str, context: &EvalContext, strip_spans: bool) -> Result<Expr, Error> {
    let tokens = tokenize(src, None)?;
    let mut parser = Parser::with_tokens(tokens);
    let mut last = NIL;
    loop {
        match parser.parse()? {
            None => return Ok(last),
            Some(expr) => {
                let expr = if strip_spans {
                    expr.without_spans()
                } else {
                    expr
                };
                last = eval(&expr, context)?;
            }
        }
    }
}

/// The default maximum number of nested procedure calls. See [`Evaluator::set_max_call_depth`].
///
/// Non-tail calls cost roughly 8–10 KiB of Rust stack per level in a debug build on macOS
/// (less in release). The default of 1000 therefore needs a large stack; hosts such as
/// `rusche-cli` evaluate on a dedicated thread with a 256 MiB stack so this limit fires as an
/// [`ErrorKind::CallDepth`] error instead of aborting. If you evaluate on a thread with a
/// smaller stack, lower the depth or raise the thread stack size.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 1000;

/// The default number of live environments that triggers automatic garbage collection.
/// See [`Evaluator::set_gc_threshold`].
pub const DEFAULT_GC_THRESHOLD: usize = 10_000;

/// A callback that reports every [`Expr`] held inside a [`Foreign`](crate::expr::Foreign)
/// object so the garbage collector can keep the environments they reference alive.
/// Register one per concrete type with [`Evaluator::register_foreign_tracer`].
pub type ForeignTracer = Box<dyn Fn(&dyn Any, &mut dyn FnMut(&Expr))>;

pub(crate) type ForeignTracers = HashMap<TypeId, ForeignTracer>;

/// Category of an evaluation error, so hosts and tests can match without parsing message text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UndefinedSymbol,
    NotCallable,
    Arity,
    Type,
    InvalidForm,
    User,
    CallDepth,
    Other,
}

/// Kind of procedure that produced a call-trace frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    Closure,
    Macro,
    Native,
}

/// One entry in an evaluation error's call trace.
///
/// Tail calls replace the active frame rather than nesting, so the trace lists only the
/// non-tail call chain that was active when the error was raised.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub name: String,
    pub kind: FrameKind,
    /// Span of the call form that entered this procedure, when available.
    pub call_site: Option<Span>,
}

/// The object that represents an expression evaluation error.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
    pub trace: Vec<Frame>,
}

impl EvalError {
    /// Creates an error with the given kind and message, and no span, help, or trace.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
            help: None,
            trace: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Bare message text without a leading span (for hosts that print their own location).
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    pub fn trace(&self) -> &[Frame] {
        &self.trace
    }
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
        Self::new(ErrorKind::Other, message)
    }
}

impl std::error::Error for EvalError {}

pub type EvalResult = Result<Expr, EvalError>;

/// The evaluation context contains the environment and other necessary state for expression evaluation.
#[derive(Clone, Debug)]
pub struct EvalContext {
    pub env: Rc<Env>,
    call_depth: Rc<Cell<usize>>,
    max_call_depth: Rc<Cell<usize>>,
    /// Active call frames for error traces. Shared across derived contexts; tail calls
    /// replace the top entry (via pop then push) so the stack stays constant-depth.
    call_trace: Rc<RefCell<Vec<Frame>>>,
}

impl EvalContext {
    /// Derives a new evaluation context from the given base context.
    /// This function can be used to create a new context within a lambda or other procedure.
    pub fn derive_from(base: &EvalContext) -> Self {
        Self {
            env: Env::derive_from(&base.env),
            call_depth: base.call_depth.clone(),
            max_call_depth: base.max_call_depth.clone(),
            call_trace: base.call_trace.clone(),
        }
    }

    pub(crate) fn push_call(&self, proc: &Proc, call_site: Option<Span>) -> Result<(), EvalError> {
        let depth = self.call_depth.get();
        let max_depth = self.max_call_depth.get();
        if depth >= max_depth {
            return Err(EvalError::new(
                ErrorKind::CallDepth,
                format!("maximum call depth ({max_depth}) exceeded"),
            ));
        }
        self.call_depth.set(depth + 1);
        self.call_trace.borrow_mut().push(Frame {
            name: proc.display_name(),
            kind: proc.frame_kind(),
            call_site,
        });
        Ok(())
    }

    pub(crate) fn pop_call(&self) {
        self.call_depth.set(self.call_depth.get() - 1);
        self.call_trace.borrow_mut().pop();
    }

    pub(crate) fn is_in_proc(&self) -> bool {
        self.call_depth.get() > 0
    }

    /// Snapshot of the active call stack (outermost first) for attaching to errors.
    pub(crate) fn trace_snapshot(&self) -> Vec<Frame> {
        self.call_trace.borrow().clone()
    }
}

/// What the evaluator does after reducing a form.
///
/// A form either produces a value, or -- when its result is the result of evaluating another
/// expression (a closure body, a macro expansion, an `if` branch) -- it hands that expression
/// back so that [`eval`] can loop on it instead of recursing. Looping is what makes tail calls
/// run in constant stack space; nothing about it leaks into [`Expr`].
pub(crate) enum Step {
    Value(Expr),
    Eval(Expr, EvalContext),
}

/// The procedure call that the [`eval`] loop is currently inside of, if any.
///
/// A tail call replaces the frame instead of nesting a new one, which is why tail calls do not
/// count towards the call depth limit. The frame is popped however the loop exits. The shared
/// [`EvalContext`] call-trace stack mirrors this: replace = pop then push.
struct CallFrame {
    context: EvalContext,
    active: bool,
}

impl CallFrame {
    fn new(context: &EvalContext) -> Self {
        Self {
            context: context.clone(),
            active: false,
        }
    }

    fn replace(&mut self, proc: &Proc, call_site: Option<Span>) -> Result<(), EvalError> {
        self.leave();
        self.context.push_call(proc, call_site)?;
        self.active = true;
        Ok(())
    }

    fn leave(&mut self) {
        if self.active {
            self.context.pop_call();
            self.active = false;
        }
    }
}

impl Drop for CallFrame {
    fn drop(&mut self) {
        self.leave();
    }
}

/// Evaluates an expression in the given context.
///
/// Evaluation runs in a loop: whenever a form reduces to another expression in tail position
/// (the last expression of a closure body, a macro expansion, or the selected branch of `if`),
/// the loop continues with that expression instead of recursing. Only non-tail positions --
/// procedure arguments, `if` conditions, non-final body expressions -- recurse on the Rust
/// stack, so tail-recursive procedures run in constant space.
///
/// On failure, span-less errors are filled from the innermost known call-form span, and a
/// call-trace frame is pushed for the active procedure (if any). Tail calls replace that
/// frame, so the returned [`EvalError::trace`] lists only the non-tail call chain.
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
    let mut expr = expr.clone();
    let mut context = context.clone();
    let mut frame = CallFrame::new(&context);

    // The innermost known source location among the forms this loop has gone through. Errors
    // raised without a span (typically by native procedures, or by code without source
    // information such as the prelude) are reported at that location. Prefer the whole call
    // form so arity errors highlight `(two 1)` rather than only the argument list.
    let mut span_hint: Option<Span> = None;

    loop {
        let cons = match &expr {
            Expr::Sym(name, span) => {
                return context.env.lookup(name).ok_or_else(|| {
                    EvalError::new(ErrorKind::UndefinedSymbol, format!("undefined symbol: `{name}`"))
                        .with_span(*span)
                });
            }
            Expr::List(List::Cons(cons), _) => cons.clone(),
            _ => return Ok(expr),
        };

        if let Some(span) = expr.span().or_else(|| cons.cdr.span()) {
            span_hint = Some(span);
        }

        match eval_form(&cons, &context, &mut frame, expr.span()) {
            Ok(Step::Value(value)) => return Ok(value),
            Ok(Step::Eval(next_expr, next_context)) => {
                expr = next_expr;
                context = next_context;
            }
            Err(mut err) => {
                if err.span.is_none() {
                    err.span = span_hint;
                }
                // Attach the active call stack once, at the outermost eval that still has
                // frames. Nested eval() calls see a non-empty shared stack and leave it alone.
                if err.trace.is_empty() {
                    let mut snapshot = context.trace_snapshot();
                    // Present innermost-first (closest to the error site).
                    snapshot.reverse();
                    err.trace = snapshot;
                }
                // Keep CallFrame alive until after the snapshot so the current frame is included.
                drop(frame);
                return Err(err);
            }
        }
    }
}

/// The forms that [`eval_form`] handles itself instead of looking them up as procedures.
///
/// `quote` and `quasiquote` must see their arguments unevaluated; the others evaluate
/// something in tail position and must hand it back to the [`eval`] loop. These names cannot
/// be shadowed by definitions.
mod form {
    pub use crate::builtin::quote::{QUASIQUOTE, QUOTE};
    pub const IF: &str = "if";
    pub const EVAL: &str = "eval";
    pub const APPLY: &str = "apply";
    pub const BEGIN: &str = "begin";
}

/// Reduces a single form `(car . cdr)` by one step.
fn eval_form(
    cons: &Cons,
    context: &EvalContext,
    frame: &mut CallFrame,
    call_site: Option<Span>,
) -> Result<Step, EvalError> {
    use crate::builtin::quote::{quasiquote, quote};
    use crate::builtin::special::apply_form;
    use crate::utils::{get_2_or_3_args, get_exact_1_arg};
    use form::*;

    let args = &cons.cdr;

    if let Expr::Sym(name, _) = &cons.car {
        match name.as_str() {
            QUOTE => return quote(name, args, context).map(Step::Value),
            QUASIQUOTE => return quasiquote(name, args, context).map(Step::Value),
            BEGIN => {
                let mut iter = args.iter().peekable();
                while let Some(expr) = iter.next() {
                    if iter.peek().is_none() {
                        return Ok(Step::Eval(expr.clone(), context.clone()));
                    }
                    eval(expr, context)?;
                }
                return Ok(Step::Value(NIL));
            }
            IF => {
                let (condition, then_clause, else_clause) = get_2_or_3_args(name, args)?;
                let cond_value = eval(condition, context)?;
                let Expr::Bool(is_true, _) = cond_value else {
                    return Err(EvalError::new(
                        ErrorKind::Type,
                        format!(
                            "`{condition}` evaluated to `{cond_value}`, expected `true` or `false`"
                        ),
                    )
                    .with_span(condition.span())
                    .with_help("conditions must be booleans; use `(not (null? x))` to test for an empty list"));
                };
                let branch = if is_true {
                    then_clause
                } else if let Some(else_clause) = else_clause {
                    else_clause
                } else {
                    return Ok(Step::Value(NIL));
                };
                return Ok(Step::Eval(branch.clone(), context.clone()));
            }
            EVAL => {
                let expr = get_exact_1_arg(name, args)?;
                return Ok(Step::Eval(eval(expr, context)?, context.clone()));
            }
            APPLY => {
                return Ok(Step::Eval(
                    apply_form(name, args, context)?,
                    context.clone(),
                ));
            }
            _ => {}
        }
    }

    let Expr::Proc(proc, _) = eval(&cons.car, context)? else {
        return Err(EvalError::new(
            ErrorKind::NotCallable,
            format!("`{}` does not evaluate to a callable", cons.car),
        )
        .with_span(cons.car.span()));
    };

    frame.replace(&proc, call_site)?;
    proc.apply(args, context)
}

/// The struct that owns the root environment and evaluation context, enforces the call depth
/// limit, and performs garbage collection.
///
/// # Call depth and tail calls
///
/// Evaluation is iterative in tail position (see [`eval`]), so tail-recursive procedures run
/// in constant stack space and do not count towards the call depth limit. Non-tail calls --
/// procedure arguments, `if` conditions, non-final `begin`/body expressions, and anything a
/// native procedure evaluates -- nest, and are capped by [`Evaluator::set_max_call_depth`].
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
                call_trace: Rc::new(RefCell::new(Vec::new())),
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

    /// Tokenizes, parses, and evaluates every top-level expression in `src`.
    ///
    /// Returns the value of the last expression, or [`NIL`] if `src` is empty.
    /// Lex, parse, and evaluation failures are reported as a unified
    /// [`crate::error::Error`].
    pub fn eval_str(&self, src: &str) -> Result<Expr, Error> {
        let result = eval_source(src, self.context(), false)?;
        self.maybe_collect_garbage(Some(&result));
        Ok(result)
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
    use crate::{error::Error, expr::intern, lexer::tokenize, parser::Parser};

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
    fn test_tail_calls_run_in_constant_depth() {
        let evaluator = Evaluator::with_prelude();
        evaluator.set_max_call_depth(50);

        // Tail calls through a closure body, `if`, `eval`, and a macro expansion all replace
        // the current call frame instead of nesting a new one.
        let src = "(define (loop n) (if (= n 0) 'done (loop (- n 1))))
                   (define (loop-eval n) (if (= n 0) 'done (eval (list 'loop-eval (- n 1)))))
                   (defmacro (my-if c a b) `(if ,c ,a ,b))
                   (define (loop-macro n) (my-if (= n 0) 'done (loop-macro (- n 1))))";
        eval_all(&evaluator, src);

        for name in ["loop", "loop-eval", "loop-macro"] {
            let result = evaluator.eval(&parse_one(&format!("({name} 10000)")));
            assert_eq!(result, Ok(intern("done")), "{name}");
        }
        assert!(!evaluator.context.is_in_proc());
    }

    #[test]
    fn test_call_depth_is_restored_after_error() {
        let evaluator = Evaluator::with_prelude();

        // An error deep inside nested non-tail calls must unwind every frame.
        eval_all(
            &evaluator,
            "(define (f n) (if (= n 0) (car '()) (+ 1 (f (- n 1)))))",
        );
        assert!(evaluator.eval(&parse_one("(f 20)")).is_err());
        assert!(!evaluator.context.is_in_proc());

        // Exceeding the limit is reported as an error, and the frames are unwound too.
        evaluator.set_max_call_depth(10);
        eval_all(&evaluator, "(define (g n) (+ 1 (g n)))");
        let err = evaluator.eval(&parse_one("(g 0)")).unwrap_err();
        assert_eq!(err.kind, ErrorKind::CallDepth, "{err}");
        assert!(
            err.message().contains("maximum call depth"),
            "{}",
            err.message()
        );
        assert!(!evaluator.context.is_in_proc());
    }

    #[test]
    fn test_begin_form() {
        let evaluator = Evaluator::with_prelude();

        assert_eq!(evaluator.eval(&parse_one("(begin)")), Ok(NIL));
        assert_eq!(evaluator.eval(&parse_one("(begin 1 2 3)")), Ok(3.into()));

        // No new scope: definitions land in the enclosing environment.
        assert_eq!(
            evaluator.eval(&parse_one("(begin (define x 1) (set! x (+ x 1)) x)")),
            Ok(2.into())
        );
        assert_eq!(evaluator.eval(&parse_one("x")), Ok(2.into()));

        // An error in a non-final form stops evaluation.
        assert!(evaluator
            .eval(&parse_one("(begin (car '()) (define y 1))"))
            .is_err());
        assert!(evaluator.eval(&parse_one("y")).is_err());

        // The last form is in tail position.
        evaluator.set_max_call_depth(50);
        eval_all(
            &evaluator,
            "(define (loop n) (begin n (if (= n 0) 'done (loop (- n 1)))))",
        );
        assert_eq!(
            evaluator.eval(&parse_one("(loop 10000)")),
            Ok(intern("done"))
        );
    }

    #[test]
    fn test_if_and_eval_forms() {
        let evaluator = Evaluator::with_prelude();

        assert_eq!(
            evaluator.eval(&parse_one("(if true 'a 'b)")),
            Ok(intern("a"))
        );
        assert_eq!(
            evaluator.eval(&parse_one("(if false 'a 'b)")),
            Ok(intern("b"))
        );
        assert_eq!(evaluator.eval(&parse_one("(if false 'a)")), Ok(NIL));
        assert!(evaluator.eval(&parse_one("(if 1 'a 'b)")).is_err());
        assert!(evaluator.eval(&parse_one("()")).is_ok());
        assert!(evaluator.eval(&parse_one("(if 1)")).is_err());
        assert!(evaluator.eval(&parse_one("(if true 2 3 4)")).is_err());

        assert_eq!(evaluator.eval(&parse_one("(eval '(+ 1 2))")), Ok(3.into()));
        assert!(evaluator.eval(&parse_one("(eval)")).is_err());
        assert!(evaluator.eval(&parse_one("(eval 1 2)")).is_err());
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

    #[test]
    fn test_eval_str() {
        let evaluator = Evaluator::with_prelude();
        assert_eq!(evaluator.eval_str("").unwrap(), NIL);
        assert_eq!(evaluator.eval_str("(+ 1 2)").unwrap(), Expr::from(3));
        assert_eq!(
            evaluator
                .eval_str("(define x 1) (set! x (+ x 1)) x")
                .unwrap(),
            Expr::from(2)
        );
        assert!(matches!(
            evaluator.eval_str("(+ 1"),
            Err(Error::Parse(crate::parser::ParseError::IncompleteExpr(_)))
        ));
        assert!(matches!(
            evaluator.eval_str("\"unterminated"),
            Err(Error::Lex(crate::lexer::LexError::IncompleteString(_)))
        ));
    }

    #[test]
    fn test_error_kinds_and_trace() {
        let evaluator = Evaluator::with_prelude();

        let err = evaluator.eval_str("undefined-sym").unwrap_err();
        let Error::Eval(e) = err else { panic!("expected eval error") };
        assert_eq!(e.kind, ErrorKind::UndefinedSymbol);
        assert!(e.message().contains("undefined symbol"));

        let err = evaluator.eval_str("(1 2)").unwrap_err();
        let Error::Eval(e) = err else { panic!("expected eval error") };
        assert_eq!(e.kind, ErrorKind::NotCallable);

        let err = evaluator.eval_str("(car 1 2)").unwrap_err();
        let Error::Eval(e) = err else { panic!("expected eval error") };
        assert_eq!(e.kind, ErrorKind::Arity);
        assert!(e.message().contains("expected 1 argument, got 2"));

        let err = evaluator.eval_str("(car 1)").unwrap_err();
        let Error::Eval(e) = err else { panic!("expected eval error") };
        assert_eq!(e.kind, ErrorKind::Type);
        assert!(e.message().contains("evaluated to `1`"));

        let err = evaluator.eval_str("(error \"boom\" 1)").unwrap_err();
        let Error::Eval(e) = err else { panic!("expected eval error") };
        assert_eq!(e.kind, ErrorKind::User);
        assert_eq!(e.message(), "boom 1");

        eval_all(
            &evaluator,
            "(define (a x) (begin (b x) 0)) (define (b x) (car x))",
        );
        let err = evaluator.eval(&parse_one("(a 42)")).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Type);
        let names: Vec<_> = err.trace().iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"a"), "{names:?}");
        assert!(names.contains(&"b") || names.contains(&"car"), "{names:?}");
    }

    fn parse_one(src: &str) -> Expr {
        Parser::with_tokens(tokenize(src, None).unwrap())
            .parse()
            .unwrap()
            .unwrap()
    }
}
