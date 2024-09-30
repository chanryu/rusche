use rusche::{
    eval::{eval, EvalContext, EvalError, EvalResult},
    expr::{Expr, NIL},
    list::List,
    utils::{eval_to_foreign, eval_to_int, get_exact_1_arg, get_exact_2_args},
};
use std::{cell::RefCell, rc::Rc};

pub fn load_vec_procs(context: &EvalContext) {
    context.env.define_native_proc("vec?", is_vec);
    context.env.define_native_proc("vec-make", make);
    context.env.define_native_proc("vec-push", push);
    context.env.define_native_proc("vec-pop", pop);
    context.env.define_native_proc("vec-get", get);
}

fn eval_to_vec(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<RefCell<Vec<Expr>>>, EvalError> {
    eval_to_foreign(proc_name, expr, context)?
        .downcast::<RefCell<Vec<Expr>>>()
        .or_else(|_| {
            Err(format!(
                "{proc_name}: {expr} does not evaluate to a vector."
            ))
        })
}

fn is_vec(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    Ok(eval_to_vec(proc_name, arg, context).is_ok().into())
}

fn make(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    Ok(Expr::Foreign(Rc::new(RefCell::new(Vec::<Expr>::new()))))
}

fn push(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;
    let vec = eval_to_vec(proc_name, arg1, context)?;
    let item = eval(arg2, context)?;
    vec.borrow_mut().push(item);
    Ok(NIL)
}

fn pop(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let vec = eval_to_vec(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    let item = vec.borrow_mut().pop();

    if let Some(item) = item {
        Ok(item)
    } else {
        Err(format!("{proc_name}: vector is empty."))
    }
}

fn get(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;
    let vec = eval_to_vec(proc_name, arg1, context)?;
    let index = eval_to_int(proc_name, "index", arg2, context)?;

    if index < 0 {
        return Err(format!("{proc_name}: index must be non-negative."));
    }

    let item = vec.borrow().get(index as usize).cloned();
    if let Some(item) = item {
        Ok(item)
    } else {
        Err(format!("{proc_name}: invalid index {index}."))
    }
}
