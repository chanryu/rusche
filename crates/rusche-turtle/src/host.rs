//! Host natives that expose [`Canvas`] to the Rusche UI script.

use std::cell::RefCell;
use std::rc::Rc;

use egui::Color32;
use rusche::{
    arity_error, cons, eval, eval_into_num, eval_into_str, get_exact_1_arg, ErrorKind, EvalContext,
    EvalError, EvalResult, Evaluator, Expr, Extension, List, NIL,
};
use rusche_egui::{Buffer, Egui};
use rusche_turtle_rs::highlight::{self, Kind};
use rusche_turtle_rs::repl::{is_complete, Entry};
use rusche_turtle_rs::{examples, status_text, Canvas};

/// Shared state for turtle host natives.
pub struct TurtleHost {
    pub canvas: Rc<RefCell<Canvas>>,
    pub egui: Egui,
}

impl Extension for TurtleHost {
    fn install(&self, evaluator: &Evaluator) {
        let env = evaluator.root_env();
        define(env, self, "turtle-frame", turtle_frame);
        define(env, self, "turtle-run", turtle_run);
        define(env, self, "turtle-repl", turtle_repl);
        define(env, self, "turtle-stop", turtle_stop);
        define(env, self, "turtle-reset", turtle_reset);
        define(env, self, "turtle-clear", turtle_clear);
        define(env, self, "turtle-speed!", turtle_speed);
        define(env, self, "turtle-running?", turtle_running);
        define(env, self, "turtle-segments", turtle_segments);
        define(env, self, "turtle-pose", turtle_pose);
        define(env, self, "turtle-pen", turtle_pen);
        define(env, self, "turtle-site", turtle_site);
        define(env, self, "turtle-transcript", turtle_transcript);
        define(env, self, "turtle-examples", turtle_examples);
        define(env, self, "turtle-output", turtle_output);
        define(env, self, "turtle-status", turtle_status);
        define(env, self, "highlight", highlight_text);
        define(env, self, "form-complete?", form_complete);
        define(env, self, "cos", cos);
        define(env, self, "sin", sin);
    }
}

fn define(
    env: &rusche::Env,
    host: &TurtleHost,
    name: &'static str,
    func: fn(&TurtleHost, &str, &List, &EvalContext) -> EvalResult,
) {
    let canvas = Rc::clone(&host.canvas);
    let egui = host.egui.clone();
    env.define_native_proc(name, move |proc_name, args, context| {
        let host = TurtleHost {
            canvas: Rc::clone(&canvas),
            egui: egui.clone(),
        };
        func(&host, proc_name, args, context)
    });
}

fn require_nil(proc_name: &str, args: &List) -> Result<(), EvalError> {
    if args.is_nil() {
        Ok(())
    } else {
        Err(arity_error(proc_name, 0..=0, args.len()))
    }
}

fn require_buffer(proc_name: &str, value: &Expr) -> Result<Rc<Buffer>, EvalError> {
    value.as_foreign::<Buffer>().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: expected a buffer, got `{value}`"),
        )
    })
}

fn turtle_frame(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let ctx = host.egui.frame_context().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: called outside a frame"),
        )
    })?;
    let running = host.canvas.borrow_mut().frame(&ctx);
    Ok(Expr::from(running))
}

fn turtle_run(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let source = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    host.canvas.borrow_mut().run_script(source);
    Ok(NIL)
}

fn turtle_repl(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let source = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    {
        let mut canvas = host.canvas.borrow_mut();
        canvas.repl.push_input(&source);
        canvas.run_repl(source);
    }
    Ok(NIL)
}

fn turtle_stop(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    host.canvas.borrow_mut().request_stop();
    Ok(NIL)
}

fn turtle_reset(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let ctx = host.egui.frame_context().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: called outside a frame"),
        )
    })?;
    host.canvas.borrow_mut().reset(&ctx);
    Ok(NIL)
}

fn turtle_clear(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    host.canvas.borrow_mut().clear();
    Ok(NIL)
}

fn turtle_speed(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let index = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    if !(index.is_finite() && index >= 0.0 && index.fract() == 0.0) {
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: expected a non-negative integer speed index"),
        ));
    }
    host.canvas.borrow_mut().set_speed(index as u32);
    Ok(NIL)
}

fn turtle_running(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    Ok(Expr::from(host.canvas.borrow().is_running()))
}

fn turtle_segments(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let canvas = host.canvas.borrow();
    let mut list = List::Nil;
    for seg in canvas.segments.iter().rev() {
        list = cons(
            Expr::from(vec![
                Expr::from(seg.x0),
                Expr::from(seg.y0),
                Expr::from(seg.x1),
                Expr::from(seg.y1),
                Expr::from(seg.color.r as f64),
                Expr::from(seg.color.g as f64),
                Expr::from(seg.color.b as f64),
                Expr::from(seg.width as f64),
            ]),
            list,
        );
    }
    Ok(Expr::List(list, None))
}

fn turtle_pose(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let pose = host.canvas.borrow().pose;
    Ok(Expr::from(vec![
        Expr::from(pose.x),
        Expr::from(pose.y),
        Expr::from(pose.heading),
        Expr::from(pose.visible),
    ]))
}

fn turtle_pen(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let pen = host.canvas.borrow().pen;
    Ok(Expr::from(vec![
        Expr::from(pen.down),
        Expr::from(pen.color.r as f64),
        Expr::from(pen.color.g as f64),
        Expr::from(pen.color.b as f64),
        Expr::from(pen.width as f64),
    ]))
}

fn turtle_site(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let script = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    match host.canvas.borrow().site_byte_range(&script) {
        Some((start, end)) => Ok(Expr::from(vec![
            Expr::from(start as f64),
            Expr::from(end as f64),
        ])),
        None => Ok(Expr::from(false)),
    }
}

fn turtle_transcript(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    _: &EvalContext,
) -> EvalResult {
    require_nil(proc_name, args)?;
    let canvas = host.canvas.borrow();
    let mut list = List::Nil;
    for entry in canvas.transcript().iter().rev() {
        let (tag, text) = match entry {
            Entry::Input(text) => ("input", text.as_str()),
            Entry::Output(text) => ("output", text.as_str()),
            Entry::Result(text) => ("result", text.as_str()),
            Entry::Error(text) => ("error", text.as_str()),
        };
        list = cons(
            Expr::from(vec![rusche::intern(tag), Expr::from(text)]),
            list,
        );
    }
    Ok(Expr::List(list, None))
}

fn turtle_examples(_: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let mut list = List::Nil;
    for example in examples::ALL.iter().rev() {
        list = cons(
            Expr::from(vec![Expr::from(example.name), Expr::from(example.source)]),
            list,
        );
    }
    Ok(Expr::List(list, None))
}

fn turtle_output(
    host: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let value = eval(arg, context)?;
    let buffer = require_buffer(proc_name, &value)?;
    let log = host.canvas.borrow().log.clone();
    if buffer.text() != log {
        buffer.set_text(log);
    }
    Ok(NIL)
}

fn turtle_status(host: &TurtleHost, proc_name: &str, args: &List, _: &EvalContext) -> EvalResult {
    require_nil(proc_name, args)?;
    let canvas = host.canvas.borrow();
    Ok(Expr::from(status_text(&canvas.pose, &canvas.pen)))
}

fn highlight_text(
    _: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values: Vec<Expr> = args
        .iter()
        .map(|a| eval(a, context))
        .collect::<Result<_, _>>()?;
    if values.len() != 2 {
        return Err(arity_error(proc_name, 2..=2, values.len()));
    }
    let text = values[0]
        .as_str()
        .ok_or_else(|| {
            EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: expected a string, got `{}`", values[0]),
            )
        })?
        .to_string();
    let dark = values[1].as_bool().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: expected a boolean, got `{}`", values[1]),
        )
    })?;
    let default = if dark {
        Color32::from_rgb(205, 214, 244)
    } else {
        Color32::from_rgb(76, 79, 105)
    };
    let mut list = List::Nil;
    for span in highlight::classify(&text).into_iter().rev() {
        if matches!(span.kind, Kind::Plain) {
            continue;
        }
        let color = highlight::color_for(span.kind, dark, default);
        list = cons(
            Expr::from(vec![
                Expr::from(span.range.start as f64),
                Expr::from(span.range.end as f64),
                Expr::from(color.r() as f64),
                Expr::from(color.g() as f64),
                Expr::from(color.b() as f64),
            ]),
            list,
        );
    }
    Ok(Expr::List(list, None))
}

fn form_complete(
    _: &TurtleHost,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(is_complete(&text)))
}

fn cos(_: &TurtleHost, proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let radians = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(radians.cos()))
}

fn sin(_: &TurtleHost, proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let radians = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(radians.sin()))
}
