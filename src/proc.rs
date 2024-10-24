use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use crate::eval::{eval, eval_tail, EvalContext, EvalError, EvalResult};
use crate::expr::NIL;
use crate::list::List;

/// The function signature for native procedures -- [`Proc::Native`].
pub type NativeFunc = fn(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult;

/// The enum that represents all procedure variants in the Rusche language.
#[derive(Clone, Debug)]
pub enum Proc {
    /// A user-defied producdure that captures outer environment.
    /// Closures can be created by the `lambda` form.
    Closure {
        name: Option<String>,
        formal_args: Vec<String>,
        body: Box<List>,
        outer_context: EvalContext,
    },

    /// A user-defied producdure that allows the user to define arbitrary functions
    /// that convert certain Lisp forms into different forms before evaluating or compiling them.
    /// Macros can be created by the `defmacro` form.
    Macro {
        name: Option<String>,
        formal_args: Vec<String>,
        body: Box<List>,
    },

    /// A native procedure that is implemented in Rust.
    Native { name: String, func: NativeFunc },
}

impl Proc {
    pub(crate) fn invoke(&self, args: &List, context: &EvalContext) -> EvalResult {
        context.push_call(self);
        let result = match self {
            Proc::Closure {
                name,
                formal_args,
                body,
                outer_context,
            } => eval_closure(
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
            } => eval_macro(name.as_deref(), formal_args, body, args, context),
            Proc::Native { name, func } => func(name, args, context),
        };
        context.pop_call();
        result
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
            (lhs, rhs) => lhs == rhs,
        }
    }
}

fn eval_closure(
    closure_name: Option<&str>,
    formal_args: &[String],
    body: &List,
    outer_context: &EvalContext,
    actual_args: &List,
    context: &EvalContext,
) -> EvalResult {
    let closure_name = closure_name.unwrap_or("unnamed-closure");
    let closure_context = EvalContext::derive_from(outer_context);
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            if let Some(name) = get_variadic_args_name(formal_arg) {
                closure_context.env.define(name, actual_args);
                break;
            }

            let expr = actual_args
                .next()
                .ok_or(EvalError::from(format!("{}: too few args", closure_name)))?;

            closure_context.env.define(formal_arg, eval(expr, context)?);
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(EvalError::from(format!("{}: too many args", closure_name)));
        }
    }

    let mut iter = body.iter().peekable();
    while let Some(expr) = iter.next() {
        if iter.peek().is_none() {
            return eval_tail(expr, &closure_context);
        } else {
            eval(expr, &closure_context)?;
        }
    }
    Ok(NIL)
}

fn eval_macro(
    macro_name: Option<&str>,
    formal_args: &[String],
    body: &List,
    actual_args: &List,
    context: &EvalContext,
) -> EvalResult {
    let macro_name = macro_name.unwrap_or("unnamed-macro");
    let macro_context = EvalContext::derive_from(context);
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            if let Some(name) = get_variadic_args_name(formal_arg) {
                macro_context.env.define(name, actual_args);
                break;
            }

            let expr = actual_args
                .next()
                .ok_or(EvalError::from(format!("{}: too few args", macro_name)))?;

            macro_context.env.define(formal_arg, expr.clone());
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(EvalError::from(format!("{}: too many args", macro_name)));
        }
    }

    let mut iter = body.iter().peekable();
    while let Some(expr) = iter.next() {
        let expanded_expr = eval(expr, &macro_context)?;
        if iter.peek().is_none() {
            return eval_tail(&expanded_expr, context);
        } else {
            eval(&expanded_expr, context)?;
        }
    }
    Ok(NIL)
}

/// Extracts the name of variadic arguments from the given name.
///
/// If the name starts with `*` and has more than one character,
/// returns the rest of the name. Otherwise, returns `None`.
///
fn get_variadic_args_name(name: &str) -> Option<&str> {
    if name.starts_with("*") && name.len() > 1 {
        Some(&name[1..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eval::Evaluator, macros::list};

    #[test]
    fn test_get_variadic_args_name() {
        assert_eq!(get_variadic_args_name("args"), None);
        assert_eq!(get_variadic_args_name("*args"), Some("args"));
        assert_eq!(get_variadic_args_name("*a"), Some("a"));
        assert_eq!(get_variadic_args_name("*"), None);
    }

    #[test]
    fn test_proc_eq() {
        let evaluator = Evaluator::default();
        let context = evaluator.context();

        let closure = Proc::Closure {
            name: Some("closure".into()),
            formal_args: vec!["a".into(), "b".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };

        let closure_same = Proc::Closure {
            name: Some("closure".into()),
            formal_args: vec!["a".into(), "b".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_eq!(closure, closure_same);

        let closure_name_diff = Proc::Closure {
            name: None,
            formal_args: vec!["a".into(), "b".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_name_diff);

        let closure_args_diff = Proc::Closure {
            name: None,
            formal_args: vec!["a".into(), "b".into(), "c".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_args_diff);

        let closure_body_diff = Proc::Closure {
            name: None,
            formal_args: vec!["a".into(), "b".into(), "c".into()],
            body: Box::new(list!(1, 2, 3, 4)),
            outer_context: context.clone(),
        };
        assert_ne!(closure, closure_body_diff);

        let closure_context_diff = Proc::Closure {
            name: None,
            formal_args: vec!["a".into(), "b".into(), "c".into()],
            body: Box::new(list!(1, 2, 3, 4)),
            outer_context: EvalContext::derive_from(&context),
        };
        assert_ne!(closure, closure_context_diff);
    }

    #[test]
    fn test_fingerprint() {
        let evaluator = Evaluator::default();
        let context = evaluator.context();

        let closure1 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: vec!["a".into(), "b".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        let closure2 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: vec!["a".into(), "b".into()],
            body: Box::new(list!(1, 2, 3)),
            outer_context: context.clone(),
        };
        let closure3 = Proc::Closure {
            name: Some("closure".into()),
            formal_args: vec!["a".into()],
            body: Box::new(list!(1, 2)),
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

        // code coverage workaround (#[coverage(off)] is unstable)
        native_fn_1("", &list!(), &context).unwrap();
        native_fn_2("", &list!(), &context).unwrap();
    }
}
