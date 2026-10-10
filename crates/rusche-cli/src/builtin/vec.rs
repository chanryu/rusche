use rusche::{
    arity_error, cons, eval, eval_into_foreign, eval_into_int, get_exact_1_arg, get_exact_2_args,
    ErrorKind, EvalContext, EvalError, EvalResult, Evaluator, Expr, List, NIL,
};

use std::{cell::RefCell, rc::Rc};

pub fn load_vec_procs(evaluator: &Evaluator) {
    let env = evaluator.root_env();
    env.define_native_proc("vec?", is_vec);
    env.define_native_proc("vec-make", vec_make);
    env.define_native_proc("vec", vec);
    env.define_native_proc("vec-push", vec_push);
    env.define_native_proc("vec-pop", vec_pop);
    env.define_native_proc("vec-get", vec_get);
    env.define_native_proc("vec-set!", vec_set);
    env.define_native_proc("vec-length", vec_length);
    env.define_native_proc("vec->list", vec_to_list);
    env.define_native_proc("list->vec", list_to_vec);

    // Let the garbage collector see closures stored inside vectors; without this, calling a
    // closure that only lives in a vector would fail after a collection.
    evaluator.register_foreign_tracer::<ExprVecRefCell>(|vec, trace| {
        vec.borrow().iter().for_each(trace);
    });
}

type ExprVecRefCell = RefCell<Vec<Expr>>;

fn eval_into_vec(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<ExprVecRefCell>, EvalError> {
    eval_into_foreign(proc_name, expr, context)?
        .downcast::<ExprVecRefCell>()
        .map_err(|_| {
            EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{expr}` does not evaluate to a vector"),
            )
            .with_span(expr.span())
        })
}

fn is_vec(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    Ok(eval_into_vec(proc_name, arg, context).is_ok().into())
}

fn vec_make(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    Ok(Expr::Foreign(Rc::new(RefCell::new(Vec::<Expr>::new()))))
}

fn vec(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut items = Vec::new();
    for arg in args.iter() {
        items.push(eval(arg, context)?);
    }
    Ok(Expr::Foreign(Rc::new(RefCell::new(items))))
}

fn vec_push(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;
    let vec = eval_into_vec(proc_name, arg1, context)?;
    let item = eval(arg2, context)?;
    vec.borrow_mut().push(item);
    Ok(NIL)
}

fn vec_pop(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let vec_expr = get_exact_1_arg(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let item = vec.borrow_mut().pop();

    if let Some(item) = item {
        Ok(item)
    } else {
        Err(
            EvalError::new(ErrorKind::Other, format!("{proc_name}: vector is empty"))
                .with_span(vec_expr.span()),
        )
    }
}

fn vec_get(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (vec_expr, index_expr) = get_exact_2_args(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let index = eval_into_int(proc_name, "index", index_expr, context)?;

    if index < 0 {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: index must be zero or positive integer"),
        )
        .with_span(index_expr.span()));
    }

    let item = vec.borrow().get(index as usize).cloned();
    if let Some(item) = item {
        Ok(item)
    } else {
        Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: index out-of-bounds {index}"),
        )
        .with_span(index_expr.span()))
    }
}

fn vec_set(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut iter = args.iter();
    let Some(vec_expr) = iter.next() else {
        return Err(arity_error(proc_name, 3..=3, 0));
    };
    let Some(index_expr) = iter.next() else {
        return Err(arity_error(proc_name, 3..=3, 1));
    };
    let Some(value_expr) = iter.next() else {
        return Err(arity_error(proc_name, 3..=3, 2));
    };
    if iter.next().is_some() {
        return Err(arity_error(proc_name, 3..=3, 3 + iter.count() + 1));
    }

    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let index = eval_into_int(proc_name, "index", index_expr, context)?;
    let value = eval(value_expr, context)?;

    if index < 0 {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: index must be zero or positive integer"),
        )
        .with_span(index_expr.span()));
    }

    let mut borrowed = vec.borrow_mut();
    let Some(slot) = borrowed.get_mut(index as usize) else {
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: index out-of-bounds {index}"),
        )
        .with_span(index_expr.span()));
    };
    *slot = value;
    Ok(NIL)
}

fn vec_length(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let vec_expr = get_exact_1_arg(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let len = vec.borrow().len() as f64;
    Ok(len.into())
}

fn vec_to_list(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let vec_expr = get_exact_1_arg(proc_name, args)?;
    let vec = eval_into_vec(proc_name, vec_expr, context)?;
    let mut list = List::Nil;
    for item in vec.borrow().iter().rev() {
        list = cons(item.clone(), list);
    }
    Ok(list.into())
}

fn list_to_vec(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let list_expr = get_exact_1_arg(proc_name, args)?;
    let list = match eval(list_expr, context)? {
        Expr::List(list, _) => list,
        other => {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: `{other}` does not evaluate to a list"),
            )
            .with_span(list_expr.span()));
        }
    };
    let items: Vec<Expr> = list.iter().cloned().collect();
    Ok(Expr::Foreign(Rc::new(RefCell::new(items))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::Evaluator;

    fn with_vec() -> Evaluator {
        let evaluator = Evaluator::with_builtin();
        load_vec_procs(&evaluator);
        evaluator
    }

    fn eval_ok(evaluator: &Evaluator, src: &str) -> String {
        match evaluator.eval_str(src) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("{src}: {e}"),
        }
    }

    fn eval_err(evaluator: &Evaluator, src: &str) -> String {
        match evaluator.eval_str(src) {
            Ok(v) => panic!("expected error, got {v}"),
            Err(e) => e.message(),
        }
    }

    #[test]
    fn vec_round_trip_and_predicates() {
        let e = with_vec();
        assert_eq!(eval_ok(&e, "(vec-length (vec-make))"), "0");
        assert_eq!(eval_ok(&e, "(vec-length (vec 1 2 3))"), "3");
        assert_eq!(eval_ok(&e, "(vec? (vec 1))"), "true");
        assert_eq!(eval_ok(&e, "(vec? 1)"), "false");
        assert_eq!(eval_ok(&e, "(vec->list (vec 1 2))"), "(1 2)");
        assert_eq!(eval_ok(&e, "(vec? (list->vec '(a b)))"), "true");
    }

    #[test]
    fn vec_mutation_and_errors() {
        let e = with_vec();
        assert_eq!(
            eval_ok(
                &e,
                "(begin
                   (define v (vec 1 2))
                   (vec-push v 3)
                   (define last (vec-pop v))
                   (vec-set! v 0 9)
                   (cons last (cons (vec-get v 0) (cons (vec-length v) ()))))"
            ),
            "(3 9 2)"
        );

        assert!(eval_err(&e, "(vec-make 1)").contains("expected 0"));
        assert!(eval_err(&e, "(vec-pop (vec-make))").contains("empty"));
        assert!(eval_err(&e, "(vec-get (vec 1) 9)").contains("out-of-bounds"));
        assert!(eval_err(&e, "(vec-get (vec 1) -1)").contains("zero or positive"));
        assert!(eval_err(&e, "(vec-set! (vec 1) 9 0)").contains("out-of-bounds"));
        assert!(eval_err(&e, "(vec-set! (vec 1) -1 0)").contains("zero or positive"));
        let msg = eval_err(&e, "(vec-get 1 0)");
        assert!(msg.contains("vector") || msg.contains("foreign"), "{msg}");
        assert!(eval_err(&e, "(list->vec 1)").contains("list"));
        assert!(eval_err(&e, "(vec-set!)").contains("expected 3"));
        assert!(eval_err(&e, "(vec-set! (vec 1))").contains("expected 3"));
        assert!(eval_err(&e, "(vec-set! (vec 1) 0)").contains("expected 3"));
        assert!(eval_err(&e, "(vec-set! (vec 1) 0 1 2)").contains("expected 3"));
    }

    #[test]
    fn vec_foreign_tracer_keeps_closures_alive() {
        let e = Evaluator::with_prelude();
        load_vec_procs(&e);
        e.set_gc_threshold(None);
        eval_ok(
            &e,
            "(define (make-counter)
               (define n 0)
               (lambda () (set! n (+ n 1)) n))
             (define v (vec (make-counter)))",
        );
        e.collect_garbage();
        assert_eq!(eval_ok(&e, "((vec-get v 0))"), "1");
    }

    #[test]
    fn wrong_foreign_type_is_rejected() {
        let e = with_vec();
        e.root_env().define("box", Expr::Foreign(Rc::new(1_i32)));
        assert_eq!(eval_ok(&e, "(vec? box)"), "false");
        let msg = eval_err(&e, "(vec-length box)");
        assert!(msg.contains("vector"), "{msg}");
    }
}
