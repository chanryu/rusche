//! Companion example for docs/tutorials/foreign.md

use std::{cell::RefCell, rc::Rc};

use rusche::{
    eval_into_foreign, get_exact_1_arg, get_exact_2_args, tokenize,
    utils::{eval_into_num, get_exact_3_args},
    EvalContext, EvalError, EvalResult, Evaluator, Expr, List, Parser, NIL,
};

struct Point {
    x: f64,
    y: f64,
}

type PointCell = RefCell<Point>;

fn eval_into_point(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<PointCell>, EvalError> {
    eval_into_foreign(proc_name, expr, context)?
        .downcast::<PointCell>()
        .map_err(|_| EvalError {
            message: format!("{proc_name}: `{expr}` does not evaluate to a point."),
            span: expr.span(),
        })
}

fn point(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (x_expr, y_expr) = get_exact_2_args(proc_name, args)?;
    let x = eval_into_num(proc_name, x_expr, context)?;
    let y = eval_into_num(proc_name, y_expr, context)?;
    Ok(Expr::Foreign(Rc::new(RefCell::new(Point { x, y }))))
}

fn point_x(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let point = eval_into_point(proc_name, arg, context)?;
    let x = point.borrow().x;
    Ok(Expr::from(x))
}

fn point_move(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (point_expr, dx_expr, dy_expr) = get_exact_3_args(proc_name, args)?;
    let point = eval_into_point(proc_name, point_expr, context)?;
    let dx = eval_into_num(proc_name, dx_expr, context)?;
    let dy = eval_into_num(proc_name, dy_expr, context)?;

    {
        let mut point = point.borrow_mut();
        point.x += dx;
        point.y += dy;
    }
    Ok(NIL)
}

fn eval_str(evaluator: &Evaluator, source: &str) -> Expr {
    let tokens = tokenize(source, None).unwrap();
    let expr = Parser::with_tokens(tokens).parse().unwrap().unwrap();
    evaluator.eval(&expr).unwrap()
}

fn main() {
    let evaluator = Evaluator::with_prelude();
    let env = evaluator.root_env();
    env.define_native_proc("point", point);
    env.define_native_proc("point-x", point_x);
    env.define_native_proc("point-move", point_move);

    eval_str(&evaluator, "(define p (point 1 2))");
    assert_eq!(eval_str(&evaluator, "(point-x p)"), Expr::from(1));
    eval_str(&evaluator, "(point-move p 3 4)");
    assert_eq!(eval_str(&evaluator, "(point-x p)"), Expr::from(4));

    println!("{}", eval_str(&evaluator, "(point-x p)"));
}
