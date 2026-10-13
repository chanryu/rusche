//! A small egui/eframe binding for reimplementing `rusche-turtle` in Rusche.
//!
//! The host owns the window. The script owns the state and rebuilds the UI from
//! one procedure each frame. [`egui::Ui`] is not a foreign object: natives borrow
//! the `Ui` that is live for the current container callback.
//!
//! Colors are lists `(r g b)` with components in `0..255`. `text-edit` mutates a
//! [`Buffer`], because Rusche strings are immutable. A painter passed to the
//! `canvas` procedure is valid only for that call.
//!
//! Coordinates are egui points: origin at the top-left of the canvas rect, `y`
//! growing downward. `paint-text` anchors the top-left of the text at `(x y)`.
//!
//! ```text
//! (define script (buffer ""))
//!
//! (define (ui)
//!   (left-panel "side" 360 (lambda ()
//!     (text-edit script 'code 'rows 30)))
//!   (central-panel (lambda ()
//!     (canvas (- (available-height) (interact-height))
//!       (lambda (painter x y w h)
//!         (fill-rect painter x y w h '(250 250 248))))))
//!   (request-repaint))
//!
//! (run-app "Rusche Turtle" 1100 720 ui)
//! ```
//!
//! `run-app` blocks until the window closes. A frame procedure that signals an
//! error is printed and the window stays open. [`Egui::run_frame`] returns that
//! error instead, so tests can drive one headless pass.
//!
//! # Procedures
//!
//! Containers take a procedure and return its value:
//! `(left-panel id size [resizable] body)`, and the same for `right-panel` and
//! `bottom-panel`; `central-panel`; `horizontal`; `right-to-left`;
//! `(scroll 'vertical|'both ['stick-to-bottom] body)`.
//!
//! `separator`, `(space n)`, `available-width`, `available-height`, `interact-height`.
//!
//! `(label text ['strong] ['small] ['weak] ['monospace] ['italics] [color])`,
//! `(button text ['disabled])` returns true when clicked,
//! `(selectable selected? text)` and `(toggle selected? text)` return the new boolean,
//! `(menu title body)`, `close-menu`, `(hover text)` on the previous widget.
//!
//! `(buffer [text])`, `buffer-text`, `buffer-set!`, `(buffer-cursor! buf 'end)`.
//! `(text-edit buf ['code] ['read-only] ['submit-on-enter] ['focus] ['rows n] [layouter])`
//! returns true when the editor has focus. `'submit-on-enter` inserts a newline on
//! Shift+Enter and leaves Enter for `(key-pressed? 'enter)`. The layouter is
//! `(lambda (text) spans)` with each span `(start end r g b)` or
//! `(start end r g b br bg bb)` in byte offsets. The optional trailing triple is a
//! background color. It may run more than once per frame and should not build widgets.
//! `(dark-mode?)` reports the active theme (usable from a layouter).
//!
//! `(canvas height (lambda (painter x y w h) ...))`,
//! `(fill-rect painter x y w h color)`,
//! `(stroke-rect painter x y w h color width)`,
//! `(line painter x0 y0 x1 y1 color width)`,
//! `(polygon painter ((x y) ...) fill stroke width)`,
//! `(circle painter x y radius fill stroke width)`,
//! `(paint-text painter x y text size color)`.
//! Pass `false` for a polygon or circle fill or stroke to skip that part.
//! A width of `0` draws no stroke.
//!
//! `(run-app title width height ui-proc)`, `request-repaint`, `dark-mode?`,
//! `(key-pressed? 'enter|'up|'down)`, `shift-down?`.

mod app;
mod buffer;
mod host;
mod paint;
mod value;
mod widgets;

pub use buffer::Buffer;

use std::rc::Rc;

use host::Host;
use rusche::{ErrorKind, EvalContext, EvalError, EvalResult, Evaluator, Expr, Extension};

/// egui natives and the context used by [`Egui::run_frame`].
///
/// Clone shares the same host. Drive the UI from one thread.
#[derive(Clone)]
pub struct Egui {
    host: Rc<Host>,
}

impl Egui {
    pub fn new() -> Self {
        Self {
            host: Rc::new(Host::new()),
        }
    }

    /// Runs `proc` once against a headless egui context of the given size, in points.
    ///
    /// The context is reused across calls, so widget state such as panel size and
    /// editor focus persists. [`Self::repaint_requested`] reports whether this pass
    /// asked for an immediate repaint.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusche::Evaluator;
    /// use rusche_egui::Egui;
    ///
    /// let evaluator = Evaluator::with_prelude();
    /// let egui = Egui::new();
    /// evaluator.install(egui.clone());
    /// let ui = evaluator
    ///     .eval_str("(lambda () (label \"Hello\"))")
    ///     .unwrap();
    /// egui.run_frame(evaluator.context(), &ui, 320.0, 240.0)
    ///     .unwrap();
    /// ```
    pub fn run_frame(
        &self,
        context: &EvalContext,
        proc: &Expr,
        width: f32,
        height: f32,
    ) -> EvalResult {
        self.run_frame_with(context, proc, width, height, Vec::new())
    }

    pub(crate) fn run_frame_with(
        &self,
        context: &EvalContext,
        proc: &Expr,
        width: f32,
        height: f32,
        events: Vec<egui::Event>,
    ) -> EvalResult {
        if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
            return Err(EvalError::new(
                ErrorKind::Other,
                "run_frame: width and height must be positive and finite".to_string(),
            ));
        }
        let ctx = self.host.egui_ctx();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            time: Some(self.host.advance_time()),
            events,
            ..Default::default()
        };
        let mut result = Ok(rusche::NIL);
        let output = ctx.run_ui(input, |ui| {
            result = self.host.run_proc(ui, context, proc);
        });
        let delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay)
            .unwrap_or(std::time::Duration::MAX);
        self.host.set_repaint_requested(delay.is_zero());
        output.drop_without_applying_deltas();
        result
    }

    /// Whether the latest [`Self::run_frame`] scheduled an immediate repaint.
    pub fn repaint_requested(&self) -> bool {
        self.host.repaint_requested()
    }

    /// The egui context for the active frame, if a widget or frame procedure is running.
    ///
    /// Hosts that need a [`egui::Context`] (for example to spawn a worker that calls
    /// `request_repaint`) use this from inside the frame procedure.
    pub fn frame_context(&self) -> Option<egui::Context> {
        if self.host.in_frame() {
            Some(self.host.egui_ctx())
        } else {
            None
        }
    }
}

impl Default for Egui {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for Egui {
    fn install(&self, evaluator: &Evaluator) {
        install(evaluator, &self.host);
    }
}

fn install(evaluator: &Evaluator, host: &Rc<Host>) {
    let env = evaluator.root_env();
    bind(env, host, "buffer", buffer::buffer);
    bind(env, host, "buffer-text", buffer::buffer_text);
    bind(env, host, "buffer-set!", buffer::buffer_set);
    bind(env, host, "buffer-cursor!", buffer::buffer_cursor);

    bind(env, host, "left-panel", widgets::left_panel);
    bind(env, host, "right-panel", widgets::right_panel);
    bind(env, host, "bottom-panel", widgets::bottom_panel);
    bind(env, host, "central-panel", widgets::central_panel);
    bind(env, host, "horizontal", widgets::horizontal);
    bind(env, host, "right-to-left", widgets::right_to_left);
    bind(env, host, "scroll", widgets::scroll);
    bind(env, host, "separator", widgets::separator);
    bind(env, host, "space", widgets::space);
    bind(env, host, "available-width", widgets::available_width);
    bind(env, host, "available-height", widgets::available_height);
    bind(env, host, "interact-height", widgets::interact_height);

    bind(env, host, "label", widgets::label);
    bind(env, host, "button", widgets::button);
    bind(env, host, "selectable", widgets::selectable);
    bind(env, host, "toggle", widgets::toggle);
    bind(env, host, "menu", widgets::menu);
    bind(env, host, "close-menu", widgets::close_menu);
    bind(env, host, "hover", widgets::hover);
    bind(env, host, "text-edit", widgets::text_edit);

    bind(env, host, "canvas", paint::canvas);
    bind(env, host, "fill-rect", paint::fill_rect);
    bind(env, host, "stroke-rect", paint::stroke_rect);
    bind(env, host, "line", paint::line);
    bind(env, host, "polygon", paint::polygon);
    bind(env, host, "circle", paint::circle);
    bind(env, host, "paint-text", paint::paint_text);

    bind(env, host, "run-app", app::run_app);
    bind(env, host, "request-repaint", request_repaint);
    bind(env, host, "dark-mode?", widgets::dark_mode);
    bind(env, host, "key-pressed?", widgets::key_pressed);
    bind(env, host, "shift-down?", widgets::shift_down);
}

fn request_repaint(
    host: &Rc<Host>,
    proc_name: &str,
    args: &rusche::List,
    _context: &EvalContext,
) -> EvalResult {
    if !args.is_nil() {
        return Err(rusche::arity_error(proc_name, 0..=0, args.len()));
    }
    host.with_ui(proc_name, |ui| {
        ui.ctx().request_repaint();
        Ok(())
    })?;
    Ok(rusche::NIL)
}

fn bind(
    env: &rusche::Env,
    host: &Rc<Host>,
    name: &'static str,
    func: fn(&Rc<Host>, &str, &rusche::List, &EvalContext) -> EvalResult,
) {
    let host = Rc::clone(host);
    env.define_native_proc(name, move |proc_name, args, context| {
        func(&host, proc_name, args, context)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::{Error, ErrorKind, Evaluator, Expr};

    fn fixture() -> (Evaluator, Egui) {
        let evaluator = Evaluator::with_prelude();
        let egui = Egui::new();
        evaluator.install(egui.clone());
        (evaluator, egui)
    }

    fn frame(egui: &Egui, evaluator: &Evaluator, src: &str) -> EvalResult {
        let proc = evaluator.eval_str(src).unwrap();
        egui.run_frame(evaluator.context(), &proc, 800.0, 600.0)
    }

    #[test]
    fn buffer_roundtrip_outside_a_frame() {
        let (evaluator, _) = fixture();
        let text = evaluator
            .eval_str(
                r#"
                (define b (buffer "hi"))
                (buffer-set! b "yo")
                (buffer-cursor! b 'end)
                (buffer-text b)
                "#,
            )
            .unwrap();
        assert_eq!(text, Expr::from("yo"));
    }

    #[test]
    fn button_outside_a_frame_errors() {
        let (evaluator, _) = fixture();
        let Error::Eval(err) = evaluator.eval_str("(button \"Go\")").unwrap_err() else {
            panic!("expected an evaluation error");
        };
        assert_eq!(err.kind, ErrorKind::Other);
        assert!(err.message().contains("outside a frame"));
    }

    #[test]
    fn widgets_build_and_report_values() {
        let (evaluator, egui) = fixture();
        evaluator.eval_str("(define n 0)").unwrap();
        let value = frame(
            &egui,
            &evaluator,
            r#"
            (lambda ()
              (left-panel "side" 200 (lambda ()
                (horizontal (lambda ()
                  (set! n 1)
                  (selectable true "Script")))
                (right-to-left (lambda ()
                  (toggle false "Help")))))
              (bottom-panel "log" 80 false (lambda ()
                (scroll 'vertical 'stick-to-bottom (lambda ()
                  (label "Output" 'strong 'small)))))
              (right-panel "help" 120 (lambda ()
                (label "api" 'monospace '(1 2 3))
                (label "hint" 'weak 'italics)))
              (central-panel (lambda ()
                (separator)
                (space 4)
                (menu "Examples" (lambda () (button "Spiral")))
                (button "Run" 'disabled)
                (hover "run the script")
                (close-menu)
                (available-height))))
            "#,
        )
        .unwrap();
        let height = value.as_num().unwrap();
        assert!(height > 100.0, "available height was {height}");
        assert_eq!(evaluator.eval_str("n").unwrap(), Expr::from(1));

        let clicked = frame(&egui, &evaluator, "(lambda () (button \"Go\"))").unwrap();
        assert_eq!(clicked, Expr::from(false));
        let selected = frame(&egui, &evaluator, "(lambda () (selectable false \"REPL\"))").unwrap();
        assert_eq!(selected, Expr::from(false));
        let toggled = frame(&egui, &evaluator, "(lambda () (toggle true \"Help\"))").unwrap();
        assert_eq!(toggled, Expr::from(true));
    }

    #[test]
    fn text_edit_layouter_and_cursor() {
        let (evaluator, egui) = fixture();
        evaluator
            .eval_str("(define buf (buffer \"ab\")) (define seen \"\")")
            .unwrap();
        let focused = frame(
            &egui,
            &evaluator,
            r#"
            (lambda ()
              (buffer-cursor! buf 'end)
              (text-edit buf 'code 'rows 4 'submit-on-enter 'focus
                (lambda (text)
                  (set! seen text)
                  '((0 2 9 8 7 255 232 140)))))
            "#,
        )
        .unwrap();
        assert!(focused.as_bool().is_some());
        assert_eq!(evaluator.eval_str("seen").unwrap(), Expr::from("ab"));
        assert_eq!(
            evaluator.eval_str("(buffer-text buf)").unwrap(),
            Expr::from("ab")
        );

        let dark = frame(&egui, &evaluator, "(lambda () (dark-mode?))").unwrap();
        assert!(dark.as_bool().is_some());

        // Invalid spans fall back to plain text instead of aborting the frame.
        frame(
            &egui,
            &evaluator,
            r#"
            (lambda ()
              (text-edit buf (lambda (text) '((0 99 1 2 3)))))
            "#,
        )
        .unwrap();
    }

    #[test]
    fn canvas_paints_and_rejects_a_stale_painter() {
        let (evaluator, egui) = fixture();
        evaluator.eval_str("(define held false)").unwrap();
        frame(
            &egui,
            &evaluator,
            r#"
            (lambda ()
              (canvas 100 (lambda (painter x y w h)
                (set! held painter)
                (fill-rect painter x y 20 10 '(250 250 248))
                (stroke-rect painter x y 20 10 '(0 0 0) 1)
                (line painter x y (+ x 10) (+ y 10) '(1 2 3) 1.5)
                (polygon painter '((0 0) (8 0) (0 8)) '(9 8 7) '(1 1 1) 1)
                (circle painter (+ x 4) (+ y 4) 3 '(2 3 4) false 0)
                (paint-text painter x y "hi" 12 '(0 0 0)))))
            "#,
        )
        .unwrap();

        let err = frame(
            &egui,
            &evaluator,
            "(lambda () (fill-rect held 0 0 8 8 '(0 0 0)))",
        )
        .unwrap_err();
        assert_eq!(err.kind, ErrorKind::Type);
        assert!(err
            .message()
            .contains("only valid during its canvas callback"));
    }

    #[test]
    fn request_repaint_is_visible_on_the_frame_output() {
        let (evaluator, egui) = fixture();
        frame(&egui, &evaluator, "(lambda () (request-repaint))").unwrap();
        assert!(egui.repaint_requested());
        // egui schedules one extra immediate pass after request_repaint.
        frame(&egui, &evaluator, "(lambda () (label \"idle\"))").unwrap();
        frame(&egui, &evaluator, "(lambda () (label \"idle\"))").unwrap();
        assert!(!egui.repaint_requested());
    }

    #[test]
    fn key_queries_see_frame_events() {
        let (evaluator, egui) = fixture();
        evaluator
            .eval_str("(define hit false) (define up false) (define shifted false)")
            .unwrap();
        let proc = evaluator
            .eval_str(
                r#"
                (lambda ()
                  (set! hit (key-pressed? 'enter))
                  (set! up (key-pressed? 'up))
                  (set! shifted (shift-down?)))
                "#,
            )
            .unwrap();
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        egui.run_frame_with(
            evaluator.context(),
            &proc,
            400.0,
            300.0,
            vec![
                egui::Event::ModifiersChanged(modifiers),
                egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
            ],
        )
        .unwrap();
        assert_eq!(evaluator.eval_str("hit").unwrap(), Expr::from(true));
        assert_eq!(evaluator.eval_str("up").unwrap(), Expr::from(false));
        assert_eq!(evaluator.eval_str("shifted").unwrap(), Expr::from(true));

        let err = frame(&egui, &evaluator, "(lambda () (key-pressed? 'space))").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Type);
    }
}
