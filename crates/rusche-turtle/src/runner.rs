//! Worker thread owning a Rusche [`Evaluator`], streaming [`TurtleEvent`]s.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;

use egui::Context;
use rusche::{Error, Evaluator, Span};

use crate::builtin::{load_turtle_procs, with_host, Host};
use crate::turtle::{Sink, Turtle, TurtleEvent};

const EVAL_STACK_SIZE: usize = 16 * 1024 * 1024;

pub struct Job {
    pub receiver: Receiver<TurtleEvent>,
    pub cancel: Arc<AtomicBool>,
    pub join: JoinHandle<()>,
}

impl Job {
    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Fresh evaluator on a large-stack thread. `ctx` is used for repaint requests.
pub fn spawn(source: String, ctx: Context) -> Job {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_worker = Arc::clone(&cancel);
    let tx_worker = tx;

    let join = std::thread::Builder::new()
        .name("rusche-turtle-eval".into())
        .stack_size(EVAL_STACK_SIZE)
        .spawn(move || {
            let host = Host {
                turtle: Turtle::new(tx_worker.clone()),
                cancel: cancel_worker,
            };

            let (mut host, result) = with_host(host, || {
                let evaluator = Evaluator::with_prelude();
                load_turtle_procs(&evaluator);
                evaluator.eval_str(&source)
            });

            if let Err(error) = result {
                let message = format_error(&error, &source, "<script>");
                host.turtle.sink_mut().emit(TurtleEvent::Error(message));
            }

            let _ = tx_worker.send(TurtleEvent::Done);
            ctx.request_repaint();
        })
        .expect("failed to spawn turtle evaluation thread");

    Job {
        receiver: rx,
        cancel,
        join,
    }
}

pub fn format_error(error: &Error, src: &str, source_name: &str) -> String {
    let mut out = format!("error: {}\n", error.message());
    if let Some(span) = error.span() {
        append_span_context(&mut out, src, source_name, &span);
    }
    if let Some(help) = error.help() {
        out.push_str(&format!("  = help: {help}\n"));
    }
    for frame in error.trace() {
        if let Some(span) = frame.call_site {
            out.push_str(&format!(
                "  = in `{}`, called at {}:{}:{}\n",
                frame.name,
                source_name,
                span.begin.line + 1,
                span.begin.column + 1
            ));
        } else {
            out.push_str(&format!("  = in `{}` (prelude)\n", frame.name));
        }
    }
    out
}

fn append_span_context(out: &mut String, src: &str, source_name: &str, span: &Span) {
    let lines: Vec<&str> = src.lines().collect();
    if span.end.line >= lines.len() {
        return;
    }
    out.push_str(&format!(
        "  --> {}:{}:{}\n",
        source_name,
        span.begin.line + 1,
        span.begin.column + 1
    ));
    let start = span.begin.line.saturating_sub(1);
    let end = span.end.line.min(lines.len() - 1);
    for (line, text) in lines.iter().enumerate().take(end + 1).skip(start) {
        out.push_str(&format!("{:>3}| {text}\n", line + 1));
        if (span.begin.line..=span.end.line).contains(&line) {
            let begin_col = if line == span.begin.line {
                span.begin.column
            } else {
                text.chars().take_while(|c| c.is_whitespace()).count()
            };
            let end_col = if line == span.end.line {
                span.end.column
            } else {
                text.chars().count()
            };
            out.push_str("   | ");
            out.push_str(&" ".repeat(begin_col));
            out.push_str(&"^".repeat(end_col.saturating_sub(begin_col).max(1)));
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::{ErrorKind, EvalError, Loc};

    #[test]
    fn format_error_includes_span_and_help() {
        let src = "(forward \"x\")\n";
        let err: Error = EvalError::new(ErrorKind::Type, "forward: expected a number")
            .with_span(Some(Span::new(Loc::new(0, 9), Loc::new(0, 12))))
            .with_help("pass a distance")
            .into();
        let text = format_error(&err, src, "<script>");
        assert!(text.contains("error: forward: expected a number"));
        assert!(text.contains("--> <script>:1:10"));
        assert!(text.contains("^^^"));
        assert!(text.contains("help: pass a distance"));
    }
}
