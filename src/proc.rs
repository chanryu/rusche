use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalContext, EvalError, EvalResult, Step};
use crate::expr::{Expr, NIL};
use crate::list::List;

/// The function signature for native procedures -- [`Proc::Native`].
pub type NativeFunc = fn(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult;

/// The formal parameters of a closure or macro.
///
/// `names` are bound positionally. `rest`, if present, is bound to a list of every remaining
/// argument -- it comes from a `*`-prefixed last parameter such as `(a b *rest)`.
#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct FormalArgs {
    pub names: Vec<String>,
    pub rest: Option<String>,
}

/// The enum that represents all procedure variants in the Rusche language.
///
/// `formal_args` and `body` are reference-counted so that looking up or calling
/// a procedure does not deep-copy its source.
#[derive(Clone, Debug)]
pub enum Proc {
    /// A user-defied producdure that captures outer environment.
    /// Closures can be created by the `lambda` form.
    Closure {
        name: Option<String>,
        formal_args: Rc<FormalArgs>,
        body: Rc<List>,
        outer_context: EvalContext,
    },

    /// A user-defied producdure that allows the user to define arbitrary functions
    /// that convert certain Lisp forms into different forms before evaluating or compiling them.
    /// Macros can be created by the `defmacro` form.
    Macro {
        name: Option<String>,
        formal_args: Rc<FormalArgs>,
        body: Rc<List>,
    },

    /// A native procedure that is implemented in Rust.
    Native { name: String, func: NativeFunc },
}

impl Proc {
    /// Applies the procedure to the unevaluated `args`.
    ///
    /// Natives produce a value directly. Closures and macros evaluate everything but their
    /// final expression and hand that expression back as [`Step::Eval`], so that the caller --
    /// the [`eval`] loop -- can continue with it in tail position.
    pub(crate) fn apply(&self, args: &List, context: &EvalContext) -> Result<Step, EvalError> {
        match self {
            Proc::Closure {
                name,
                formal_args,
                body,
                outer_context,
            } => apply_closure(
                name.as_deref(),
                formal_args,
                body,
                outer_context,
                args,
                context,
            ),
            Proc::Macro {
                name,
                formal_args,
                body,
            } => apply_macro(name.as_deref(), formal_args, body, args, context),
            Proc::Native { name, func } => func(name, args, context).map(Step::Value),
        }
    }

    pub(crate) fn badge(&self) -> String {
        match self {
            Proc::Closure { name, .. } => {
                format!("proc/closure:{}", name.as_deref().unwrap_or("unnamed"),)
            }
            Proc::Macro { name, .. } => {
                format!("proc/macro:{}", name.as_deref().unwrap_or("unnamed"),)
            }
            Proc::Native { name, .. } => {
                format!("proc/native:{}", name)
            }
        }
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        match self {
            Proc::Closure {
                formal_args,
                body,
                outer_context,
                ..
            } => {
                formal_args.hash(&mut hasher);
                body.to_string().hash(&mut hasher);
                Rc::as_ptr(&outer_context.env).hash(&mut hasher);
            }
            Proc::Macro {
                formal_args, body, ..
            } => {
                formal_args.hash(&mut hasher);
                body.to_string().hash(&mut hasher);
            }
            Proc::Native { func, .. } => {
                func.hash(&mut hasher);
            }
        }

        format!("{}:{:x}", self.badge(), hasher.finish())
    }
}

impl PartialEq for Proc {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Proc::Closure {
                    name: name1,
                    formal_args: formal_args1,
                    body: body1,
                    outer_context: outer_context1,
                },
                Proc::Closure {
                    name: name2,
                    formal_args: formal_args2,
                    body: body2,
                    outer_context: outer_context2,
                },
            ) => {
                name1 == name2
                    && formal_args1 == formal_args2
                    && body1 == body2
                    && Rc::ptr_eq(&outer_context1.env, &outer_context2.env)
            }
            (
                Proc::Macro {
                    name: name1,
                    formal_args: formal_args1,
                    body: body1,
                },
                Proc::Macro {
                    name: name2,
                    formal_args: formal_args2,
                    body: body2,
                },
            ) => name1 == name2 && formal_args1 == formal_args2 && body1 == body2,
            (
                Proc::Native {
                    name: name1,
                    func: func1,
                },
                Proc::Native {
                    name: name2,
                    func: func2,
                },
            ) => name1 == name2 && std::ptr::fn_addr_eq(*func1, *func2),
            _ => false,
        }
    }
}

/// Binds `actual_args` to `formal_args` in `env`, passing each argument through `value`
/// (evaluation for closures, identity for macros).
fn bind_args(
    proc_name: &str,
    formal_args: &FormalArgs,
    actual_args: &List,
    env: &Env,
    mut value: impl FnMut(&Expr) -> EvalResult,
) -> Result<(), EvalError> {
    let mut actual_args = actual_args.iter();

    for name in &formal_args.names {
        let expr = actual_args
            .next()
            .ok_or_else(|| EvalError::from(format!("{proc_name}: too few args")))?;
        env.define(name, value(expr)?);
    }

    match &formal_args.rest {
        Some(rest) => {
            let values = actual_args.map(value).collect::<Result<Vec<_>, _>>()?;
            env.define(rest, List::from(values));
        }
        None if actual_args.next().is_some() => {
            return Err(EvalError::from(format!("{proc_name}: too many args")));
        }
        None => {}
    }

    Ok(())
}

fn apply_closure(
    closure_name: Option<&str>,
    formal_args: &FormalArgs,
    body: &List,
    outer_context: &EvalContext,
    actual_args: &List,
    context: &EvalContext,
) -> Result<Step, EvalError> {
    let closure_name = closure_name.unwrap_or("unnamed-closure");
    let closure_context = EvalContext::derive_from(outer_context);
    bind_args(
        closure_name,
        formal_args,
        actual_args,
        &closure_context.env,
        |expr| eval(expr, context),
    )?;

    let mut iter = body.iter().peekable();
    while let Some(expr) = iter.next() {
        if iter.peek().is_none() {
            return Ok(Step::Eval(expr.clone(), closure_context));
        }
        eval(expr, &closure_context)?;
    }
    Ok(Step::Value(NIL))
}

fn apply_macro(
    macro_name: Option<&str>,
    formal_args: &FormalArgs,
    body: &List,
    actual_args: &List,
    context: &EvalContext,
) -> Result<Step, EvalError> {
    let macro_name = macro_name.unwrap_or("unnamed-macro");
    let macro_context = EvalContext::derive_from(context);
    bind_args(
        macro_name,
        formal_args,
        actual_args,
        &macro_context.env,
        |expr| Ok(expr.clone()),
    )?;

    let mut iter = body.iter().peekable();
    while let Some(expr) = iter.next() {
        let expanded_expr = eval(expr, &macro_context)?;
        if iter.peek().is_none() {
            return Ok(Step::Eval(expanded_expr, context.clone()));
        }
        eval(&expanded_expr, context)?;
    }
    Ok(Step::Value(NIL))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eval::Evaluator, macros::list};

    fn formal_args(names: &[&str]) -> Rc<FormalArgs> {
        Rc::new(FormalArgs {
            names: names.iter().map(|s| s.to_string()).collect(),
            rest: None,
        })
    }

    #[test]
    fn test_proc_eq() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let closure = Proc::Closure {
            name: Some("closure".into()),
            formal_args: formal_args(&["a", "b"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };

        let closure_same = Proc::Closure {
            name: Some("closure".into()),
            formal_args: formal_args(&["a", "b"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_eq!(closure, closure_same);

        let closure_name_diff = Proc::Closure {
            name: None,
            formal_args: formal_args(&["a", "b"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_name_diff);

        let closure_args_diff = Proc::Closure {
            name: None,
            formal_args: formal_args(&["a", "b", "c"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_args_diff);

        let closure_body_diff = Proc::Closure {
            name: None,
            formal_args: formal_args(&["a", "b", "c"]),
            body: Rc::new(list!(1, 2, 3, 4)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_body_diff);

        let closure_context_diff = Proc::Closure {
            name: None,
            formal_args: formal_args(&["a", "b", "c"]),
            body: Rc::new(list!(1, 2, 3, 4)),
            outer_context: EvalContext::derive_from(context),
        };
        assert_ne!(closure, closure_context_diff);
    }

    #[test]
    fn test_proc_eq_native_and_macro() {
        // Regression: comparing two non-closure procs used to recurse forever.
        fn native_fn_1(_: &str, _: &List, _: &EvalContext) -> EvalResult {
            Ok(NIL)
        }
        fn native_fn_2(_: &str, _: &List, _: &EvalContext) -> EvalResult {
            Ok(NIL)
        }

        let native = |name: &str, func: NativeFunc| Proc::Native {
            name: name.into(),
            func,
        };
        assert_eq!(native("a", native_fn_1), native("a", native_fn_1));
        assert_ne!(native("a", native_fn_1), native("b", native_fn_1));
        assert_ne!(native("a", native_fn_1), native("a", native_fn_2));

        let macro_ = |name: &str| Proc::Macro {
            name: Some(name.into()),
            formal_args: formal_args(&["x"]),
            body: Rc::new(list!(1)),
        };
        assert_eq!(macro_("m"), macro_("m"));
        assert_ne!(macro_("m"), macro_("n"));

        // mixed kinds are never equal
        assert_ne!(native("m", native_fn_1), macro_("m"));

        let evaluator = Evaluator::new();
        let closure = Proc::Closure {
            name: Some("m".into()),
            formal_args: formal_args(&["x"]),
            body: Rc::new(list!(1)),
            outer_context: evaluator.context().clone(),
        };
        assert_ne!(closure, macro_("m"));
        assert_ne!(closure, native("m", native_fn_1));

        // code coverage workaround (#[coverage(off)] is unstable)
        native_fn_1("", &list!(), evaluator.context()).unwrap();
        native_fn_2("", &list!(), evaluator.context()).unwrap();
    }

    #[test]
    fn test_fingerprint() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        let closure1 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: formal_args(&["a", "b"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        let closure2 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: formal_args(&["a", "b"]),
            body: Rc::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        let closure3 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: formal_args(&["a"]),
            body: Rc::new(list!(1, 2)),
            outer_context: context.clone(),
        };
        assert_eq!(closure1.fingerprint(), closure2.fingerprint());
        assert_ne!(closure1.fingerprint(), closure3.fingerprint());

        fn native_fn_1(_: &str, _: &List, _: &EvalContext) -> EvalResult {
            Ok(NIL)
        }
        fn native_fn_2(_: &str, _: &List, _: &EvalContext) -> EvalResult {
            Ok(NIL)
        }

        let native1 = Proc::Native {
            name: "native".into(),
            func: native_fn_1,
        };
        let native1_1 = Proc::Native {
            name: "native".into(),
            func: native_fn_1,
        };
        let native2 = Proc::Native {
            name: "native".into(),
            func: native_fn_2,
        };
        assert_eq!(native1.fingerprint(), native1_1.fingerprint());
        assert_ne!(native1.fingerprint(), native2.fingerprint());

        let macro_ = |names: &[&str], body: List| Proc::Macro {
            name: Some("m".into()),
            formal_args: formal_args(names),
            body: Rc::new(body),
        };
        assert_eq!(
            macro_(&["x"], list!(1)).fingerprint(),
            macro_(&["x"], list!(1)).fingerprint()
        );
        assert_ne!(
            macro_(&["x"], list!(1)).fingerprint(),
            macro_(&["y"], list!(1)).fingerprint()
        );
        assert_ne!(
            macro_(&["x"], list!(1)).fingerprint(),
            macro_(&["x"], list!(2)).fingerprint()
        );
        assert!(macro_(&["x"], list!(1))
            .fingerprint()
            .starts_with("proc/macro:m:"));

        // code coverage workaround (#[coverage(off)] is unstable)
        native_fn_1("", &list!(), context).unwrap();
        native_fn_2("", &list!(), context).unwrap();
    }

    fn eval_str(evaluator: &Evaluator, src: &str) -> EvalResult {
        use crate::{lexer::tokenize, parser::Parser};

        let mut parser = Parser::with_tokens(tokenize(src, None).unwrap());
        let mut last = Ok(NIL);
        while let Some(expr) = parser.parse().unwrap() {
            last = evaluator.eval(&expr);
        }
        last
    }

    #[test]
    fn test_apply_closure_body() {
        let evaluator = Evaluator::with_builtin();

        // An empty body evaluates to ().
        assert_eq!(eval_str(&evaluator, "((lambda ()))"), Ok(NIL));

        // Every body expression runs; the last one is the result.
        let src = "(define x 0)
                   ((lambda () (set! x (num-add x 1)) (set! x (num-add x 1)) x))";
        assert_eq!(eval_str(&evaluator, src), Ok(2.into()));

        // An error in a non-final body expression stops evaluation.
        let src = "(define y 0)
                   ((lambda () (car '()) (set! y 1)))";
        assert!(eval_str(&evaluator, src).is_err());
        assert_eq!(eval_str(&evaluator, "y"), Ok(0.into()));
    }

    #[test]
    fn test_apply_macro_body() {
        let evaluator = Evaluator::with_builtin();

        // An empty body expands to nothing and evaluates to ().
        assert_eq!(eval_str(&evaluator, "(defmacro (empty)) (empty)"), Ok(NIL));

        // Each body form is expanded and then evaluated in the caller's environment; the
        // expansion of the last one is the result.
        let src = "(defmacro (m) '(define z 41) '(num-add z 1))
                   (m)";
        assert_eq!(eval_str(&evaluator, src), Ok(42.into()));
        assert_eq!(eval_str(&evaluator, "z"), Ok(41.into()));

        // An error in a non-final expansion stops evaluation.
        let src = "(defmacro (bad) '(car '()) '(define w 1))
                   (bad)";
        assert!(eval_str(&evaluator, src).is_err());
        assert!(eval_str(&evaluator, "w").is_err());
    }
}
