//! Persistent worker thread owning a Rusche [`Evaluator`], streaming [`TurtleEvent`]s.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use egui::Context;
use rusche::{Error, Evaluator, Span};

use crate::builtin::{load_turtle_procs, Host};
use crate::turtle::{Origin, Turtle, TurtleEvent, TurtleSnap};

const EVAL_STACK_SIZE: usize = 16 * 1024 * 1024;

enum Request {
    Eval {
        source: String,
        origin: Origin,
    },
    /// Align the worker turtle with the UI after Stop discarded queued drawing.
    Sync(TurtleSnap),
}

/// A long-lived evaluation session. Dropping it disconnects the request channel and the
/// worker exits after the in-flight request (if any) finishes or is cancelled.
pub struct Session {
    requests: Sender<Request>,
    pub events: Receiver<TurtleEvent>,
    cancel: Arc<AtomicBool>,
}

impl Session {
    /// Spawns a worker with a fresh evaluator. `ctx` is used for repaint requests.
    pub fn spawn(ctx: Context) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<Request>();
        let (evt_tx, evt_rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_worker = Arc::clone(&cancel);
        let evt_worker = evt_tx;

        std::thread::Builder::new()
            .name("rusche-turtle-eval".into())
            .stack_size(EVAL_STACK_SIZE)
            .spawn(move || {
                let host = Rc::new(RefCell::new(Host {
                    turtle: Turtle::new(evt_worker.clone()),
                }));
                let evaluator = Evaluator::with_prelude();
                evaluator.set_interrupt(Some(cancel_worker.clone()));
                load_turtle_procs(&evaluator, &host);

                while let Ok(request) = req_rx.recv() {
                    match request {
                        Request::Sync(snap) => host.borrow_mut().turtle.restore(&snap),
                        Request::Eval { source, origin } => {
                            cancel_worker.store(false, Ordering::Relaxed);
                            let _ = evt_worker.send(TurtleEvent::Started(origin));

                            let source_name = match origin {
                                Origin::Script => "<script>",
                                Origin::Repl => "<repl>",
                            };

                            match evaluator.eval_str(&source) {
                                Ok(value) => {
                                    if origin == Origin::Repl {
                                        let _ =
                                            evt_worker.send(TurtleEvent::Result(value.to_string()));
                                    }
                                }
                                Err(error) => {
                                    let message = format_error(&error, &source, source_name);
                                    let _ = evt_worker.send(TurtleEvent::Error(message));
                                }
                            }

                            let _ = evt_worker.send(TurtleEvent::Done);
                            ctx.request_repaint();
                        }
                    }
                }
            })
            .expect("failed to spawn turtle evaluation thread");

        Self {
            requests: req_tx,
            events: evt_rx,
            cancel,
        }
    }

    /// Queue `source` for evaluation. Returns `false` if the worker is gone.
    pub fn eval(&self, source: String, origin: Origin) -> bool {
        self.requests.send(Request::Eval { source, origin }).is_ok()
    }

    /// Queue a pose/pen restore so the next eval continues from the UI turtle.
    pub fn sync_turtle(&self, snap: TurtleSnap) -> bool {
        self.requests.send(Request::Sync(snap)).is_ok()
    }

    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
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
    use std::time::{Duration, Instant};

    fn drain_until_done(session: &Session, timeout: Duration) -> Vec<TurtleEvent> {
        let deadline = Instant::now() + timeout;
        let mut events = Vec::new();
        loop {
            match session.events.try_recv() {
                Ok(TurtleEvent::Done) => {
                    events.push(TurtleEvent::Done);
                    return events;
                }
                Ok(event) => events.push(event),
                Err(mpsc::TryRecvError::Empty) => {
                    if Instant::now() >= deadline {
                        panic!("timed out waiting for Done; got {events:?}");
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("session disconnected; got {events:?}");
                }
            }
        }
    }

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

    #[test]
    fn session_shares_definitions_across_requests() {
        let session = Session::spawn(Context::default());
        assert!(session.eval("(define (double x) (* x 2))".into(), Origin::Repl));
        let first = drain_until_done(&session, Duration::from_secs(5));
        assert!(matches!(
            first.first(),
            Some(TurtleEvent::Started(Origin::Repl))
        ));
        assert!(first.iter().any(|e| matches!(e, TurtleEvent::Result(_))));
        assert!(matches!(first.last(), Some(TurtleEvent::Done)));

        assert!(session.eval("(double 21)".into(), Origin::Repl));
        let second = drain_until_done(&session, Duration::from_secs(5));
        assert!(matches!(
            second.iter().find(|e| matches!(e, TurtleEvent::Result(_))),
            Some(TurtleEvent::Result(s)) if s == "42"
        ));
    }

    #[test]
    fn session_repl_error_emits_error_event() {
        let session = Session::spawn(Context::default());
        assert!(session.eval("(car 1)".into(), Origin::Repl));
        let events = drain_until_done(&session, Duration::from_secs(5));
        assert!(matches!(
            events.first(),
            Some(TurtleEvent::Started(Origin::Repl))
        ));
        assert!(events.iter().any(|e| matches!(e, TurtleEvent::Error(_))));
        assert!(!events.iter().any(|e| matches!(e, TurtleEvent::Result(_))));
        assert!(matches!(events.last(), Some(TurtleEvent::Done)));
    }

    #[test]
    fn session_script_success_skips_result() {
        let session = Session::spawn(Context::default());
        assert!(session.eval("(+ 1 2)".into(), Origin::Script));
        let events = drain_until_done(&session, Duration::from_secs(5));
        assert!(matches!(
            events.first(),
            Some(TurtleEvent::Started(Origin::Script))
        ));
        assert!(!events.iter().any(|e| matches!(e, TurtleEvent::Result(_))));
        assert!(matches!(events.last(), Some(TurtleEvent::Done)));
    }

    #[test]
    fn session_sync_then_forward_continues_from_snap() {
        let session = Session::spawn(Context::default());
        assert!(session.sync_turtle(TurtleSnap {
            x: 40.0,
            y: -10.0,
            heading: 0.0,
            visible: true,
            pen_down: true,
            color: crate::turtle::Color::BLUE,
            width: 2.0,
        }));
        // Sync is silent; give the worker a moment, then move.
        std::thread::sleep(Duration::from_millis(20));
        assert!(session.eval("(forward 10)".into(), Origin::Repl));
        let events = drain_until_done(&session, Duration::from_secs(5));
        let line = events.iter().find_map(|e| match e {
            TurtleEvent::Line(s) => Some(s),
            _ => None,
        });
        let line = line.expect("expected a line after sync");
        assert!((line.x0 - 40.0).abs() < 1e-9);
        assert!((line.y0 - (-10.0)).abs() < 1e-9);
        assert!((line.x1 - 50.0).abs() < 1e-9);
        assert_eq!(line.color, crate::turtle::Color::BLUE);
    }
}
