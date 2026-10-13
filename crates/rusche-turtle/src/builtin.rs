//! Turtle natives. Each procedure closes over a shared [`Host`].

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use rand::Rng;
use rusche::eval_into_num;
use rusche::{
    arity_error, cons, eval, get_exact_1_arg, get_exact_2_args, ErrorKind, EvalContext, EvalError,
    EvalResult, Evaluator, Expr, Extension, List, NIL,
};

use crate::turtle::{Color, Turtle, TurtleEvent};

pub struct Host {
    pub turtle: Turtle<Sender<TurtleEvent>>,
}

/// Runs `f` against the captured host and tags the events `f` emits with the script form
/// currently being evaluated (see [`TurtleEvent::Site`]).
fn with_turtle<R>(
    host: &RefCell<Host>,
    context: &EvalContext,
    f: impl FnOnce(&mut Host) -> Result<R, EvalError>,
) -> Result<R, EvalError> {
    let mut host = host.borrow_mut();
    host.turtle.set_site(context.call_site());
    f(&mut host)
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

fn nil_op(
    host: &RefCell<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
    f: impl FnOnce(&mut Turtle<Sender<TurtleEvent>>),
) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(host, context, |host| {
        f(&mut host.turtle);
        Ok(NIL)
    })
}

fn format_write_args(args: &List, context: &EvalContext) -> Result<String, EvalError> {
    let mut text = String::new();
    for expr in args.iter() {
        let value = eval(expr, context)?;
        if let Some(s) = value.as_str() {
            text.push_str(s);
        } else {
            text.push_str(&value.to_string());
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

type Native = fn(&Rc<RefCell<Host>>, &str, &List, &EvalContext) -> EvalResult;

fn define(env: &rusche::Env, host: &Rc<RefCell<Host>>, name: &'static str, func: Native) {
    let host = Rc::clone(host);
    env.define_native_proc(name, move |proc_name, args, context| {
        func(&host, proc_name, args, context)
    });
}

/// Turtle natives plus the `repeat` macro.
pub struct TurtleProcs(pub Rc<RefCell<Host>>);

impl Extension for TurtleProcs {
    fn install(&self, evaluator: &Evaluator) {
        let env = evaluator.root_env();
        let host = &self.0;
        define(env, host, "forward", forward);
        define(env, host, "fd", forward);
        define(env, host, "back", back);
        define(env, host, "bk", back);
        define(env, host, "left", left);
        define(env, host, "lt", left);
        define(env, host, "right", right);
        define(env, host, "rt", right);
        define(env, host, "goto", goto);
        define(env, host, "setheading", setheading);
        define(env, host, "seth", setheading);
        define(env, host, "home", home);
        define(env, host, "penup", penup);
        define(env, host, "pu", penup);
        define(env, host, "pendown", pendown);
        define(env, host, "pd", pendown);
        define(env, host, "pencolor", pencolor);
        define(env, host, "pensize", pensize);
        define(env, host, "clear", clear);
        define(env, host, "hideturtle", hideturtle);
        define(env, host, "showturtle", showturtle);
        define(env, host, "position", position);
        define(env, host, "heading", heading);
        define(env, host, "pen?", is_pen);
        define(env, host, "write", write);
        define(env, host, "writeln", writeln);
        define(env, host, "random", random);
        evaluator
            .eval_prelude(PRELUDE)
            .expect("turtle prelude must evaluate");
    }
}

fn forward(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let dist = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.forward(dist);
        Ok(NIL)
    })
}

fn back(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let dist = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.back(dist);
        Ok(NIL)
    })
}

fn left(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let deg = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.left(deg);
        Ok(NIL)
    })
}

fn right(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let deg = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.right(deg);
        Ok(NIL)
    })
}

fn goto(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (x_expr, y_expr) = get_exact_2_args(proc_name, args)?;
    let x = eval_into_num(proc_name, x_expr, context)?;
    let y = eval_into_num(proc_name, y_expr, context)?;
    with_turtle(host, context, |host| {
        host.turtle.goto(x, y);
        Ok(NIL)
    })
}

fn setheading(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let deg = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.set_heading(deg);
        Ok(NIL)
    })
}

fn home(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.home())
}

fn penup(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.pen_up())
}

fn pendown(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.pen_down())
}

fn pencolor(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let color = match args.len() {
        1 => {
            let arg = get_exact_1_arg(proc_name, args)?;
            let value = eval(arg, context)?;
            if let Some(name) = value.as_str() {
                Color::from_name(name).ok_or_else(|| {
                    EvalError::new(
                        ErrorKind::Other,
                        format!("{proc_name}: unknown color name `{name}`"),
                    )
                    .with_span(arg.span())
                })?
            } else if let Some(n) = value.as_num() {
                let c = Color::clamp_channel(n);
                Color { r: c, g: c, b: c }
            } else {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: expected a color name or rgb numbers, got `{value}`"),
                )
                .with_span(arg.span()));
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
    with_turtle(host, context, |host| {
        host.turtle.set_color(color);
        Ok(NIL)
    })
}

fn pensize(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let size = unary_num(proc_name, args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.set_width(size as f32);
        Ok(NIL)
    })
}

fn clear(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.clear())
}

fn hideturtle(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.hide())
}

fn showturtle(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    nil_op(host, proc_name, args, context, |t| t.show())
}

fn position(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(host, context, |host| {
        Ok(cons(
            Expr::from(host.turtle.x),
            cons(Expr::from(host.turtle.y), List::Nil),
        )
        .into())
    })
}

fn heading(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(host, context, |host| Ok(Expr::from(host.turtle.heading)))
}

fn is_pen(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(host, context, |host| Ok(host.turtle.pen_down.into()))
}

fn write(host: &Rc<RefCell<Host>>, _: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = format_write_args(args, context)?;
    with_turtle(host, context, |host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn writeln(host: &Rc<RefCell<Host>>, _: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut text = format_write_args(args, context)?;
    text.push('\n');
    with_turtle(host, context, |host| {
        host.turtle.print(text);
        Ok(NIL)
    })
}

fn random(
    host: &Rc<RefCell<Host>>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    require_nil(proc_name, args)?;
    with_turtle(host, context, |_| {
        Ok(rand::thread_rng().gen::<f64>().into())
    })
}

#[cfg(test)]
fn eval_with_vec_sink(src: &str) -> Result<(Vec<TurtleEvent>, Expr), rusche::Error> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let host = Rc::new(RefCell::new(Host {
        turtle: Turtle::new(tx),
    }));
    let evaluator = Evaluator::with_prelude();
    evaluator.install(TurtleProcs(Rc::clone(&host)));
    let result = evaluator.eval_str(src);
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
        let (events, result) =
            eval_with_vec_sink("(begin (repeat 4 (fd 10) (rt 90)) (list (position) (heading)))")
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

    fn sites(events: &[TurtleEvent]) -> Vec<String> {
        events
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Site(Some(span)) => Some(span.to_string()),
                TurtleEvent::Site(None) => Some("?".into()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn site_events_track_script_forms() {
        // Each turtle call is preceded by the span of its own call form; repeated calls
        // from the same form (inside `repeat`) emit the site once.
        let src = "(repeat 2 (fd 10) (rt 90))\n(penup)";
        let (events, _) = eval_with_vec_sink(src).unwrap();
        assert_eq!(
            sites(&events),
            ["1:11-17", "1:19-25", "1:11-17", "1:19-25", "2:1-7"]
        );
        assert!(matches!(events[0], TurtleEvent::Site(_)));
        assert!(matches!(events[1], TurtleEvent::Line(_)));

        // Calls inside a user-defined procedure point at the form in the body.
        let src = "(define (side) (fd 10))\n(side)";
        let (events, _) = eval_with_vec_sink(src).unwrap();
        assert_eq!(sites(&events), ["1:16-22"]);

        // A runtime-built call falls back to the innermost enclosing source call, `(go)`.
        let src = "(define (go) (apply fd '(10)) (rt 90))\n(go)";
        let (events, _) = eval_with_vec_sink(src).unwrap();
        assert_eq!(sites(&events), ["2:1-4", "1:31-37"]);
    }

    #[test]
    fn bundled_examples_evaluate() {
        for example in crate::examples::ALL {
            eval_with_vec_sink(example.source).unwrap_or_else(|e| panic!("{}: {e}", example.name));
        }
    }

    #[test]
    fn interrupt_stops_a_loop_with_no_turtle_calls() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let (tx, rx) = mpsc::channel();
        let host = Rc::new(RefCell::new(Host {
            turtle: Turtle::new(tx),
        }));
        let evaluator = Evaluator::with_prelude();
        let flag = Arc::new(AtomicBool::new(false));
        evaluator.set_interrupt(Some(Arc::clone(&flag)));
        evaluator.install(TurtleProcs(host));
        evaluator.eval_str("(define (spin) (spin))").unwrap();
        flag.store(true, Ordering::Relaxed);
        let result = evaluator.eval_str("(spin)");
        drop(rx);
        let message = result.unwrap_err().message();
        assert!(message.contains("evaluation interrupted"), "{message}");
    }
}
