//! Turtle natives. Host state is `thread_local` because `NativeFunc` is a fn pointer.

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

pub struct Host {
    pub turtle: Turtle<Sender<TurtleEvent>>,
    pub cancel: Arc<AtomicBool>,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

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

fn require_nil(proc_name: &str, args: &List) -> Result<(), EvalError> {
    if args.is_nil() {
        Ok(())
    } else {
        Err(arity_error(proc_name, 0..=0, args.len()))
    }
}

fn unary_num(proc_name: &str, args: &List, context: &EvalContext) -> Result<f64, EvalError> {
    eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)
}

fn nil_op(proc_name: &str, args: &List, f: impl FnOnce(&mut Turtle<Sender<TurtleEvent>>)) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(|host| {
        f(&mut host.turtle);
        Ok(NIL)
    })
}

fn format_write_args(args: &List, context: &EvalContext) -> Result<String, EvalError> {
    let mut text = String::new();
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(s, _) => text.push_str(&s),
            other => text.push_str(&other.to_string()),
        }
    }
    Ok(text)
}

const PRELUDE: &str = r#"
(defmacro (repeat n *body)
    `(let ((i 0) (limit ,n))
        (while (< i limit)
            ,@body
            (set! i (+ i 1)))))
"#;

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
    let dist = unary_num(proc_name, args, context)?;
    with_turtle(|host| {
        host.turtle.forward(dist);
        Ok(NIL)
    })
}

fn back(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let dist = unary_num(proc_name, args, context)?;
    with_turtle(|host| {
        host.turtle.back(dist);
        Ok(NIL)
    })
}

fn left(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let deg = unary_num(proc_name, args, context)?;
    with_turtle(|host| {
        host.turtle.left(deg);
        Ok(NIL)
    })
}

fn right(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let deg = unary_num(proc_name, args, context)?;
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
    let deg = unary_num(proc_name, args, context)?;
    with_turtle(|host| {
        host.turtle.set_heading(deg);
        Ok(NIL)
    })
}

fn home(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.home())
}

fn penup(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.pen_up())
}

fn pendown(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.pen_down())
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
                        format!("{proc_name}: expected a color name or rgb numbers, got `{other}`"),
                    )
                    .with_span(arg.span()));
                }
            }
        }
        3 => {
            let mut iter = args.iter();
            Color {
                r: Color::clamp_channel(eval_into_num(proc_name, iter.next().unwrap(), context)?),
                g: Color::clamp_channel(eval_into_num(proc_name, iter.next().unwrap(), context)?),
                b: Color::clamp_channel(eval_into_num(proc_name, iter.next().unwrap(), context)?),
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
    let size = unary_num(proc_name, args, context)?;
    with_turtle(|host| {
        host.turtle.set_width(size as f32);
        Ok(NIL)
    })
}

fn clear(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.clear())
}

fn hideturtle(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.hide())
}

fn showturtle(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    nil_op(proc_name, args, |t| t.show())
}

fn position(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(|host| {
        Ok(cons(
            Expr::from(host.turtle.x),
            cons(Expr::from(host.turtle.y), List::Nil),
        )
        .into())
    })
}

fn heading(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(|host| Ok(Expr::from(host.turtle.heading)))
}

fn is_pen(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(|host| Ok(host.turtle.pen_down.into()))
}

fn write(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = format_write_args(args, context)?;
    with_turtle(|host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn writeln(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut text = format_write_args(args, context)?;
    text.push('\n');
    with_turtle(|host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn random(proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    // Touches the host so cancel is checked (tight loops can call `random`).
    require_nil(proc_name, args)?;
    with_turtle(|_| Ok(rand::thread_rng().gen::<f64>().into()))
}

#[cfg(test)]
fn eval_with_vec_sink(src: &str) -> Result<(Vec<TurtleEvent>, Expr), rusche::Error> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let (host, result) = with_host(
        Host {
            turtle: Turtle::new(tx),
            cancel: Arc::new(AtomicBool::new(false)),
        },
        || {
            let evaluator = Evaluator::with_prelude();
            load_turtle_procs(&evaluator);
            evaluator.eval_str(src)
        },
    );
    drop(host);
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

    fn nums(expr: &Expr) -> Vec<f64> {
        match expr {
            Expr::Num(n, _) => vec![*n],
            Expr::List(list, _) => list.iter().flat_map(nums).collect(),
            _ => panic!("expected numbers in {expr}"),
        }
    }

    #[test]
    fn repeat_square() {
        let (events, result) = eval_with_vec_sink(
            "(begin (repeat 4 (fd 10) (rt 90)) (list (position) (heading)))",
        )
        .unwrap();
        assert_eq!(lines(&events).len(), 4);
        assert!(events.iter().any(|e| matches!(e, TurtleEvent::Turn { .. })));
        let n = nums(&result);
        assert!(n[0].abs() < 1e-6 && n[1].abs() < 1e-6);
        assert!((n[2] - 90.0).abs() < 1e-6);
    }

    #[test]
    fn position_and_heading_queries() {
        let (events, result) = eval_with_vec_sink(
            "(begin (penup) (fd 50) (rt 90) (list (position) (heading) (pen?)))",
        )
        .unwrap();
        assert!(lines(&events).is_empty());
        let Expr::List(outer, _) = result else {
            panic!("expected list");
        };
        let mut it = outer.iter();
        let n = nums(it.next().unwrap());
        assert!(n[0].abs() < 1e-9 && (n[1] - 50.0).abs() < 1e-9);
        assert!((nums(it.next().unwrap())[0]).abs() < 1e-9);
        assert!(matches!(it.next(), Some(Expr::Bool(false, _))));
    }

    #[test]
    fn pencolor_name_and_rgb() {
        eval_with_vec_sink(r#"(begin (pencolor "red") (pencolor 10 20 30) (pensize 4))"#).unwrap();
    }

    #[test]
    fn arity_errors() {
        assert!(eval_with_vec_sink("(forward)")
            .unwrap_err()
            .message()
            .contains("expected 1"));
        assert!(eval_with_vec_sink("(home 1)")
            .unwrap_err()
            .message()
            .contains("expected 0"));
        assert!(eval_with_vec_sink(r#"(pencolor "mauve")"#)
            .unwrap_err()
            .message()
            .contains("unknown color"));
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
        for example in crate::examples::ALL {
            eval_with_vec_sink(example.source)
                .unwrap_or_else(|e| panic!("{}: {e}", example.name));
        }
    }

    #[test]
    fn cancel_aborts_turtle_call() {
        let (tx, rx) = mpsc::channel();
        let (_host, result) = with_host(
            Host {
                turtle: Turtle::new(tx),
                cancel: Arc::new(AtomicBool::new(true)),
            },
            || {
                let evaluator = Evaluator::with_prelude();
                load_turtle_procs(&evaluator);
                evaluator.eval_str("(forward 10)")
            },
        );
        drop(rx);
        assert!(result.unwrap_err().message().contains("stopped"));
    }
}
