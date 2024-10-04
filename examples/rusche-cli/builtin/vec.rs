use rusche::{
    eval::{eval, EvalContext, EvalError,  EvalResult},
    expr::{Expr, NIL},
    list::List,
    utils::{eval_into_foreign, eval_into_int, get_exact_1_arg, get_exact_2_args},
};
use std::{cell::RefCell, rc::Rc};

pub fn load_vec_procs(context: &EvalContext) {
    context.env.define_native_proc("vec?", is_vec);
    context.env.define_native_proc("vec-make", make);
    context.env.define_native_proc("vec-push", push);
    context.env.define_native_proc("vec-pop", pop);
    context.env.define_native_proc("vec-get", get);
}

type ExprVecRefCell = RefCell<Vec<Expr>>;

fn eval_into_vec(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<ExprVecRefCell>, EvalError> {
    eval_into_foreign(proc_name, expr, context)?
        .downcast::<ExprVecRefCell>()
        .or_else(|_| {
            Err(EvalError {
                message: format!("{proc_name}: `{expr}` does not evaluate to a vector."),
                span: expr.span(),
            })
        })
}

fn is_vec(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    Ok(eval_into_vec(proc_name, arg, context).is_ok().into())
}

fn make(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    Ok(Expr::Foreign(Rc::new(RefCell::new(Vec::<Expr>::new()))))
}

fn push(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;
    let vec = eval_into_vec(proc_name, arg1, context)?;
    let item = eval(arg2, context)?;
    vec.borrow_mut().push(item);
    Ok(NIL)
}

fn pop(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let vec_expr = get_exact_1_arg(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let item = vec.borrow_mut().pop();

    if let Some(item) = item {
        Ok(item)
    } else {
        Err(EvalError {
            message: format!("{proc_name}: vector is empty."),
            span: vec_expr.span(),
        })
    }
}

fn get(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (vec_expr, index_expr) = get_exact_2_args(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let index = eval_into_int(proc_name, "index", index_expr, context)?;

    if index < 0 {
        return Err(EvalError {
            message: format!("{proc_name}: index must be zero or positive integer."),
            span: index_expr.span(),
        });
    }

    let item = vec.borrow().get(index as usize).cloned();
    if let Some(item) = item {
        Ok(item)
    } else {
        Err(EvalError {
            message: format!("{proc_name}: index out-of-bounds {index}."),
            span: index_expr.span(),
        })
    }
}
