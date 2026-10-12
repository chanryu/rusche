//! Turtle natives registered on a Rusche [`Evaluator`].
//!
//! `NativeFunc` is a function pointer, so host state lives in a
//! `thread_local` installed by the worker before evaluation.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use rand::Rng;
use rusche::utils::eval_into_num;
use rusche::{
    arity_error, cons, eval, get_exact_1_arg, get_exact_2_args, ErrorKind, EvalContext, EvalError,
    EvalResult, Evaluator, Expr, List, NIL,
};

use crate::turtle::{Color, Turtle, TurtleEvent};

/// Shared host state for the current worker thread.
pub struct Host {
    pub turtle: Turtle<Sender<TurtleEvent>>,
    pub cancel: Arc<AtomicBool>,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

/// Install host state for the duration of `f`. Restores the previous value after.
pub fn with_host<R>(host: Host, f: impl FnOnce() -> R) -> (Host, R) {
    HOST.with(|slot| {
        let previous = slot.borrow_mut().replace(host);
        let result = f();
        let host = slot.borrow_mut().take().expect("host missing after eval");
        *slot.borrow_mut() = previous;
        (host, result)
    })
}

fn with_turtle<R>(f: impl FnOnce(&mut Host) -> Result<R, EvalError>) -> Result<R, EvalError> {
    HOST.with(|slot| {
        let mut borrow = slot.borrow_mut();
        let host = borrow
            .as_mut()
            .ok_or_else(|| EvalError::new(ErrorKind::Other, "turtle host is not installed"))?;
        if host.cancel.load(Ordering::Relaxed) {
            return Err(EvalError::new(ErrorKind::Other, "script stopped"));
        }
        f(host)
    })
}

const PRELUDE: &str = r#"
(defmacro (repeat n *body)
    `(let ((i 0) (limit ,n))
        (while (< i limit)
            ,@body
            (set! i (+ i 1)))))
"#;

/// Register turtle / I/O natives and load the turtle prelude macro.
pub fn load_turtle_procs(evaluator: &Evaluator) {
    let env = evaluator.root_env();
    env.define_native_proc("forward", forward);
    env.define_native_proc("fd", forward);
    env.define_native_proc("back", back);
    env.define_native_proc("bk", back);
    env.define_native_proc("left", left);
    env.define_native_proc("lt", left);
    env.define_native_proc("right", right);
    env.define_native_proc("rt", right);
    env.define_native_proc("goto", goto);
    env.define_native_proc("setheading", setheading);
    env.define_native_proc("seth", setheading);
    env.define_native_proc("home", home);
    env.define_native_proc("penup", penup);
    env.define_native_proc("pu", penup);
    env.define_native_proc("pendown", pendown);
    env.define_native_proc("pd", pendown);
    env.define_native_proc("pencolor", pencolor);
    env.define_native_proc("pensize", pensize);
    env.define_native_proc("clear", clear);
    env.define_native_proc("hideturtle", hideturtle);
    env.define_native_proc("showturtle", showturtle);
    env.define_native_proc("position", position);
    env.define_native_proc("heading", heading);
    env.define_native_proc("pen?", is_pen);
    env.define_native_proc("write", write);
    env.define_native_proc("writeln", writeln);
    env.define_native_proc("random", random);

    evaluator
        .eval_str(PRELUDE)
        .expect("turtle prelude must evaluate");
}

fn forward(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let dist = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.forward(dist);
        Ok(NIL)
    })
}

fn back(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let dist = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.back(dist);
        Ok(NIL)
    })
}

fn left(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let deg = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.left(deg);
        Ok(NIL)
    })
}

fn right(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let deg = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.right(deg);
        Ok(NIL)
    })
}

fn goto(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (x_expr, y_expr) = get_exact_2_args(proc_name, args)?;
    let x = eval_into_num(proc_name, x_expr, context)?;
    let y = eval_into_num(proc_name, y_expr, context)?;
    with_turtle(|host| {
        host.turtle.goto(x, y);
        Ok(NIL)
    })
}

fn setheading(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let deg = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.set_heading(deg);
        Ok(NIL)
    })
}

fn home(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.home();
        Ok(NIL)
    })
}

fn penup(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.pen_up();
        Ok(NIL)
    })
}

fn pendown(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.pen_down();
        Ok(NIL)
    })
}

fn pencolor(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let color = match args.len() {
        1 => {
            let arg = get_exact_1_arg(proc_name, args)?;
            match eval(arg, context)? {
                Expr::Str(name, _) => Color::from_name(&name).ok_or_else(|| {
                    EvalError::new(
                        ErrorKind::Other,
                        format!("{proc_name}: unknown color name `{name}`"),
                    )
                    .with_span(arg.span())
                })?,
                Expr::Num(n, _) => {
                    let c = Color::clamp_channel(n);
                    Color { r: c, g: c, b: c }
                }
                other => {
                    return Err(EvalError::new(
                        ErrorKind::Type,
                        format!(
                            "{proc_name}: expected a color name or rgb numbers, got `{other}`"
                        ),
                    )
                    .with_span(arg.span()));
                }
            }
        }
        3 => {
            let mut iter = args.iter();
            let r = eval_into_num(proc_name, iter.next().unwrap(), context)?;
            let g = eval_into_num(proc_name, iter.next().unwrap(), context)?;
            let b = eval_into_num(proc_name, iter.next().unwrap(), context)?;
            Color {
                r: Color::clamp_channel(r),
                g: Color::clamp_channel(g),
                b: Color::clamp_channel(b),
            }
        }
        n => {
            return Err(EvalError::new(
                ErrorKind::Arity,
                format!("{proc_name}: expected 1 or 3 arguments, got {n}"),
            ));
        }
    };
    with_turtle(|host| {
        host.turtle.set_color(color);
        Ok(NIL)
    })
}

fn pensize(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let size = eval_into_num(proc_name, arg, context)?;
    with_turtle(|host| {
        host.turtle.set_width(size as f32);
        Ok(NIL)
    })
}

fn clear(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.clear();
        Ok(NIL)
    })
}

fn hideturtle(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.hide();
        Ok(NIL)
    })
}

fn showturtle(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        host.turtle.show();
        Ok(NIL)
    })
}

fn position(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| {
        let x = host.turtle.x;
        let y = host.turtle.y;
        Ok(cons(Expr::from(x), cons(Expr::from(y), List::Nil)).into())
    })
}

fn heading(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| Ok(Expr::from(host.turtle.heading)))
}

fn is_pen(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    with_turtle(|host| Ok(host.turtle.pen_down.into()))
}

fn write(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut text = String::new();
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(s, _) => text.push_str(&s),
            other => text.push_str(&other.to_string()),
        }
    }
    with_turtle(|host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn writeln(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut text = String::new();
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(s, _) => text.push_str(&s),
            other => text.push_str(&other.to_string()),
        }
    }
    text.push('\n');
    with_turtle(|host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn random(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    // Check cancel even for random so tight loops that call it can stop.
    with_turtle(|_| Ok(rand::thread_rng().gen::<f64>().into()))
}

/// Test helper: evaluate `src` against a turtle that records events in a `Vec`.
#[cfg(test)]
pub fn eval_with_vec_sink(src: &str) -> Result<(Vec<TurtleEvent>, Expr), rusche::Error> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let host = Host {
        turtle: Turtle::new(tx),
        cancel,
    };

    let (host, result) = with_host(host, || {
        let evaluator = Evaluator::with_prelude();
        load_turtle_procs(&evaluator);
        evaluator.eval_str(src)
    });
    drop(host); // close sender
    let events: Vec<_> = rx.try_iter().collect();
    result.map(|expr| (events, expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turtle::Segment;
    use std::sync::mpsc;

    fn lines(events: &[TurtleEvent]) -> Vec<&Segment> {
        events
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Line(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn repeat_square() {
        let (events, result) = eval_with_vec_sink(
            "(begin (repeat 4 (fd 10) (rt 90)) (list (position) (heading)))",
        )
        .unwrap();
        assert_eq!(lines(&events).len(), 4);
        assert!(events.iter().any(|e| matches!(e, TurtleEvent::Turn { .. })));
        let Expr::List(outer, _) = result else {
            panic!("expected list");
        };
        let mut it = outer.iter();
        let Expr::List(pos, _) = it.next().unwrap().clone() else {
            panic!("expected position");
        };
        let mut pos_it = pos.iter();
        let Expr::Num(x, _) = pos_it.next().unwrap().clone() else {
            panic!("x");
        };
        let Expr::Num(y, _) = pos_it.next().unwrap().clone() else {
            panic!("y");
        };
        let Expr::Num(h, _) = it.next().unwrap().clone() else {
            panic!("heading");
        };
        assert!(x.abs() < 1e-6, "x={x}");
        assert!(y.abs() < 1e-6, "y={y}");
        assert!((h - 90.0).abs() < 1e-6, "heading={h}");
    }

    #[test]
    fn position_and_heading_queries() {
        let (events, result) = eval_with_vec_sink(
            "(begin (penup) (fd 50) (rt 90) (list (position) (heading) (pen?)))",
        )
        .unwrap();
        assert!(lines(&events).is_empty());
        // Position is a nested list ((x y) heading pen?)
        let Expr::List(outer, _) = result else {
            panic!("expected list");
        };
        let mut it = outer.iter();
        let Expr::List(pos, _) = it.next().unwrap().clone() else {
            panic!("expected position list");
        };
        let mut pos_it = pos.iter();
        let Expr::Num(x, _) = pos_it.next().unwrap().clone() else {
            panic!("x");
        };
        let Expr::Num(y, _) = pos_it.next().unwrap().clone() else {
            panic!("y");
        };
        let Expr::Num(h, _) = it.next().unwrap().clone() else {
            panic!("heading");
        };
        let Expr::Bool(pen, _) = it.next().unwrap().clone() else {
            panic!("pen?");
        };
        assert!(x.abs() < 1e-9, "x={x}");
        assert!((y - 50.0).abs() < 1e-9, "y={y}");
        assert!((h - 0.0).abs() < 1e-9, "heading={h}");
        assert!(!pen);
    }

    #[test]
    fn pencolor_name_and_rgb() {
        eval_with_vec_sink(r#"(begin (pencolor "red") (pencolor 10 20 30) (pensize 4))"#)
            .unwrap();
    }

    #[test]
    fn arity_errors() {
        let err = eval_with_vec_sink("(forward)").unwrap_err();
        assert!(err.message().contains("expected 1"), "{}", err.message());
        let err = eval_with_vec_sink("(home 1)").unwrap_err();
        assert!(err.message().contains("expected 0"), "{}", err.message());
        let err = eval_with_vec_sink(r#"(pencolor "mauve")"#).unwrap_err();
        assert!(err.message().contains("unknown color"), "{}", err.message());
    }

    #[test]
    fn write_goes_to_log_events() {
        let (events, _) = eval_with_vec_sink(r#"(writeln "hi " 1)"#).unwrap();
        let printed: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Print(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(printed, ["hi 1\n"]);
    }

    #[test]
    fn bundled_examples_evaluate() {
        for (name, src) in [
            ("squares", include_str!("../examples/squares.rsc")),
            ("spiral", include_str!("../examples/spiral.rsc")),
            ("star", include_str!("../examples/star.rsc")),
            ("tree", include_str!("../examples/tree.rsc")),
            ("colors", include_str!("../examples/colors.rsc")),
            ("koch", include_str!("../examples/koch.rsc")),
        ] {
            eval_with_vec_sink(src).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    #[test]
    fn cancel_aborts_turtle_call() {
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(true));
        let host = Host {
            turtle: Turtle::new(tx),
            cancel,
        };
        let (_host, result) = with_host(host, || {
            let evaluator = Evaluator::with_prelude();
            load_turtle_procs(&evaluator);
            evaluator.eval_str("(forward 10)")
        });
        drop(rx);
        let err = result.unwrap_err();
        assert!(err.message().contains("stopped"), "{}", err.message());
    }

}
