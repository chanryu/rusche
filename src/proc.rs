use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use crate::eval::{eval, eval_tail, EvalContext, EvalError, EvalErrorCode, EvalResult};
use crate::expr::NIL;
use crate::list::List;

pub type NativeFunc = fn(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult;

#[derive(Clone, Debug)]
pub enum Proc {
    Closure {
        name: Option<String>,
        formal_args: Vec<String>,
        body: Box<List>,
        outer_context: EvalContext,
    },
    Macro {
        name: Option<String>,
        formal_args: Vec<String>,
        body: Box<List>,
    },
    Native {
        name: String,
        func: NativeFunc,
    },
}

impl Proc {
    pub fn invoke(&self, args: &List, context: &EvalContext) -> EvalResult {
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

    pub fn identity(&self) -> String {
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

        format!("{}:{:x}", self.identity(), hasher.finish())
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
    formal_args: &Vec<String>,
    body: &List,
    outer_context: &EvalContext,
    actual_args: &List,
    context: &EvalContext,
) -> EvalResult {
    let closure_name = closure_name.unwrap_or("closure");
    let closure_context = EvalContext::derive_from(&outer_context);
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            if let Some(name) = get_variadic_args_name(formal_arg) {
                closure_context.env.define(name, actual_args);
                break;
            }

            let expr = actual_args.next().ok_or(EvalError {
                code: EvalErrorCode::ArityMismatch,
                message: format!("{}: too few args", closure_name),
                span: None,
            })?;

            closure_context.env.define(formal_arg, eval(expr, context)?);
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(EvalError {
                code: EvalErrorCode::ArityMismatch,
                message: format!("{}: too many args", closure_name),
                span: None,
            });
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
    formal_args: &Vec<String>,
    body: &List,
    actual_args: &List,
    context: &EvalContext,
) -> EvalResult {
    let macro_name = macro_name.unwrap_or("macro");
    let macro_context = EvalContext::derive_from(context);
    let mut formal_args = formal_args.iter();
    let mut actual_args = actual_args.iter();

    loop {
        if let Some(formal_arg) = formal_args.next() {
            if let Some(name) = get_variadic_args_name(formal_arg) {
                macro_context.env.define(name, actual_args);
                break;
            }

            let expr = actual_args.next().ok_or(EvalError {
                code: EvalErrorCode::ArityMismatch,
                message: format!("{}: too few args", macro_name),
                span: None,
            })?;

            macro_context.env.define(formal_arg, expr.clone());
        } else {
            if actual_args.next().is_none() {
                break;
            }
            return Err(EvalError {
                code: EvalErrorCode::ArityMismatch,
                message: format!("{}: too many args", macro_name),
                span: None,
            });
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

    #[test]
    fn test_get_variadic_args_name() {
        assert_eq!(get_variadic_args_name("args"), None);
        assert_eq!(get_variadic_args_name("*args"), Some("args"));
        assert_eq!(get_variadic_args_name("*a"), Some("a"));
        assert_eq!(get_variadic_args_name("*"), None);
    }
}
