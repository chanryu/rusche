//! Animated turtle canvas and evaluation session.

use std::ops::Range;
use std::sync::mpsc::TryRecvError;

use egui::Context;
use rusche::Span;

use crate::repl::{Entry, ReplState};
use crate::runner::Session;
use crate::turtle::{Color, Origin, Segment, TurtleEvent, TurtleSnap, DEFAULT_PEN_WIDTH};

/// Turtle pose in turtle coordinates (origin center, y up). Heading 0° = east, 90° = north.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
    pub visible: bool,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            heading: 90.0,
            visible: true,
        }
    }
}

/// Pen the canvas is currently showing. Matches a fresh turtle until a script changes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenState {
    pub down: bool,
    pub color: Color,
    pub width: f32,
}

impl Default for PenState {
    fn default() -> Self {
        Self {
            down: true,
            color: Color::BLACK,
            width: DEFAULT_PEN_WIDTH,
        }
    }
}

struct Walk {
    x1: f64,
    y1: f64,
    pen: Option<(Color, f32)>,
    end_heading: Option<f64>,
    end_visible: Option<bool>,
}

enum Motion {
    Walk(Walk),
    /// Remaining degrees; positive = counter-clockwise.
    Turn {
        remaining: f64,
    },
}

/// Five animation paces, slowest first. The budget is pixels of travel per frame;
/// turns spend the same budget at [`TURN_DEG_PER_PX`] degrees per pixel.
pub const SPEEDS: &[(&str, f64)] = &[
    ("Crawl", 0.5),
    ("Walk", 1.25),
    ("Trot", 5.0),
    ("Run", 32.0),
    ("Dash", 128.0),
];
const TURN_DEG_PER_PX: f64 = 2.0;
const WALK_EPS: f64 = 0.25;
const TURN_EPS: f64 = 0.25;

fn face(dx: f64, dy: f64) -> f64 {
    dy.atan2(dx).to_degrees().rem_euclid(360.0)
}

/// Pose, pen, segments, and the worker session that drives them.
pub struct Canvas {
    session: Option<Session>,
    pub segments: Vec<Segment>,
    pending: Vec<TurtleEvent>,
    motion: Option<Motion>,
    pub pose: Pose,
    pub pen: PenState,
    /// Evaluation requests whose `Done` has not yet been applied.
    in_flight: usize,
    /// Origin of the request currently being drained from `pending`.
    current_origin: Origin,
    /// Index into [`SPEEDS`], slowest first.
    pub speed: u32,
    /// Script form whose effects are currently being shown on the canvas.
    pub active_site: Option<Span>,
    pub log: String,
    /// REPL transcript, input buffer, and history.
    pub repl: ReplState,
    /// After Stop, discard drawing events until the cancelled eval's `Done` arrives
    /// so the worker cannot pull the UI ahead of the visible turtle again.
    drop_drawing: bool,
}

impl Canvas {
    /// Creates a canvas with no worker. Call [`Self::ensure_session`] before evaluating.
    pub fn new() -> Self {
        Self {
            session: None,
            segments: Vec::new(),
            pending: Vec::new(),
            motion: None,
            pose: Pose::default(),
            pen: PenState::default(),
            in_flight: 0,
            current_origin: Origin::Script,
            speed: 2, // Trot, the middle pace
            active_site: None,
            log: String::new(),
            repl: ReplState::default(),
            drop_drawing: false,
        }
    }

    /// Spawns the worker if it is not already running. `ctx` receives repaint requests.
    pub fn ensure_session(&mut self, ctx: &Context) {
        if self.session.is_none() {
            self.session = Some(Session::spawn(ctx.clone()));
        }
    }

    pub fn is_running(&self) -> bool {
        self.in_flight > 0 || !self.pending.is_empty() || self.motion.is_some()
    }

    pub fn set_speed(&mut self, index: u32) {
        if (index as usize) < SPEEDS.len() {
            self.speed = index;
        }
    }

    /// Poll the worker and advance one speed budget. Returns true while animation or
    /// evaluation is still in progress (the host should request a repaint).
    pub fn frame(&mut self, ctx: &Context) -> bool {
        self.ensure_session(ctx);
        self.poll_session();
        self.apply_pending();
        if !self.is_running() {
            self.active_site = None;
        }
        self.is_running()
    }

    pub fn run_script(&mut self, source: String) {
        self.submit(source, Origin::Script);
    }

    pub fn run_repl(&mut self, source: String) {
        self.submit(source, Origin::Repl);
    }

    pub fn request_stop(&mut self) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        session.request_stop();
        self.pending.retain(|e| !is_drawing_event(e));
        self.motion = None;
        self.active_site = None;
        let _ = session.sync_turtle(self.snap());
        self.drop_drawing = self.in_flight > 0;
    }

    pub fn reset(&mut self, ctx: &Context) {
        if let Some(session) = self.session.take() {
            session.request_stop();
        }
        self.session = Some(Session::spawn(ctx.clone()));
        self.in_flight = 0;
        self.pending.clear();
        self.motion = None;
        self.active_site = None;
        self.drop_drawing = false;
        self.segments.clear();
        self.pose = Pose::default();
        self.pen = PenState::default();
        self.log.clear();
        self.repl.clear_transcript();
    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.pending.retain(|e| !is_drawing_event(e));
        self.motion = None;
        self.pose = Pose::default();
        self.pen = PenState::default();
        if let Some(session) = self.session.as_ref() {
            let _ = session.sync_turtle(self.snap());
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(session) = self.session.take() {
            session.request_stop();
        }
    }

    /// Byte range of the form currently being animated, if it still fits `script`.
    pub fn site_byte_range(&self, script: &str) -> Option<(usize, usize)> {
        let span = self.active_site.as_ref()?;
        let range = span_to_byte_range(script, span)?;
        Some((range.start, range.end))
    }

    pub fn transcript(&self) -> &[Entry] {
        &self.repl.transcript
    }

    fn snap(&self) -> TurtleSnap {
        TurtleSnap {
            x: self.pose.x,
            y: self.pose.y,
            heading: self.pose.heading,
            visible: self.pose.visible,
            pen_down: self.pen.down,
            color: self.pen.color,
            width: self.pen.width,
        }
    }

    fn submit(&mut self, source: String, origin: Origin) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        if session.eval(source, origin) {
            self.in_flight += 1;
        }
    }

    fn poll_session(&mut self) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        loop {
            match session.events.try_recv() {
                Ok(event) => {
                    if self.drop_drawing && is_drawing_event(&event) {
                        continue;
                    }
                    self.pending.push(event);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    fn apply_pending(&mut self) {
        if self.motion.is_none() && self.pending.is_empty() {
            return;
        }

        let mut budget = SPEEDS[self.speed as usize].1;
        while budget > 0.0 {
            if self.motion.is_none() {
                while self.motion.is_none() && !self.pending.is_empty() {
                    let event = self.pending.remove(0);
                    self.begin_or_apply(event);
                }
                if self.motion.is_none() {
                    break;
                }
            }
            budget = self.advance_motion(budget);
        }
    }

    fn begin_or_apply(&mut self, event: TurtleEvent) {
        match event {
            TurtleEvent::Line(seg) => {
                let dx = seg.x1 - seg.x0;
                let dy = seg.y1 - seg.y0;
                if dx.hypot(dy) > WALK_EPS {
                    self.pose.heading = face(dx, dy);
                }
                self.pose.x = seg.x0;
                self.pose.y = seg.y0;
                let mut walk = Walk {
                    x1: seg.x1,
                    y1: seg.y1,
                    pen: Some((seg.color, seg.width)),
                    end_heading: None,
                    end_visible: None,
                };
                self.absorb_trailing_move(&mut walk);
                self.motion = Some(Motion::Walk(walk));
            }
            TurtleEvent::Move {
                x,
                y,
                heading,
                visible,
            } => {
                let dx = x - self.pose.x;
                let dy = y - self.pose.y;
                if dx.hypot(dy) > WALK_EPS {
                    self.pose.heading = face(dx, dy);
                    self.motion = Some(Motion::Walk(Walk {
                        x1: x,
                        y1: y,
                        pen: None,
                        end_heading: Some(heading),
                        end_visible: Some(visible),
                    }));
                } else {
                    self.pose = Pose {
                        x,
                        y,
                        heading,
                        visible,
                    };
                }
            }
            TurtleEvent::Turn { delta } if delta.abs() > TURN_EPS => {
                self.motion = Some(Motion::Turn { remaining: delta });
            }
            TurtleEvent::Turn { .. } => {}
            other => self.apply_event_instant(other),
        }
    }

    fn absorb_trailing_move(&mut self, walk: &mut Walk) {
        let Some(TurtleEvent::Move {
            x,
            y,
            heading,
            visible,
        }) = self.pending.first().cloned()
        else {
            return;
        };
        if (x - walk.x1).hypot(y - walk.y1) <= WALK_EPS {
            self.pending.remove(0);
            walk.end_heading = Some(heading);
            walk.end_visible = Some(visible);
        }
    }

    fn finish_walk(&mut self, walk: Walk) {
        if let Some((color, width)) = walk.pen {
            let dx = walk.x1 - self.pose.x;
            let dy = walk.y1 - self.pose.y;
            if dx.hypot(dy) > 1e-9 {
                self.push_segment(Segment {
                    x0: self.pose.x,
                    y0: self.pose.y,
                    x1: walk.x1,
                    y1: walk.y1,
                    color,
                    width,
                });
            }
        }
        self.pose.x = walk.x1;
        self.pose.y = walk.y1;
        if let Some(h) = walk.end_heading {
            self.pose.heading = h;
        }
        if let Some(v) = walk.end_visible {
            self.pose.visible = v;
        }
        self.motion = None;
    }

    fn push_segment(&mut self, seg: Segment) {
        if let Some(last) = self.segments.last_mut() {
            let joins = (last.x1 - seg.x0).hypot(last.y1 - seg.y0) <= 1e-6;
            if joins && last.color == seg.color && (last.width - seg.width).abs() < 1e-6 {
                let adx = last.x1 - last.x0;
                let ady = last.y1 - last.y0;
                let bdx = seg.x1 - seg.x0;
                let bdy = seg.y1 - seg.y0;
                let alen = adx.hypot(ady);
                let blen = bdx.hypot(bdy);
                let cross = adx * bdy - ady * bdx;
                let dot = adx * bdx + ady * bdy;
                if alen > 1e-9 && blen > 1e-9 && cross.abs() <= 1e-6 * alen * blen && dot > 0.0 {
                    last.x1 = seg.x1;
                    last.y1 = seg.y1;
                    return;
                }
            }
        }
        self.segments.push(seg);
    }

    fn advance_motion(&mut self, budget: f64) -> f64 {
        match self.motion {
            Some(Motion::Walk(_)) => self.advance_walk(budget),
            Some(Motion::Turn { .. }) => self.advance_turn(budget),
            None => budget,
        }
    }

    fn advance_walk(&mut self, mut budget: f64) -> f64 {
        while budget > 0.0 {
            let Some(Motion::Walk(walk)) = self.motion.as_ref() else {
                return budget;
            };
            let dx = walk.x1 - self.pose.x;
            let dy = walk.y1 - self.pose.y;
            let dist = dx.hypot(dy);
            let pen = walk.pen;
            let walk = Walk {
                x1: walk.x1,
                y1: walk.y1,
                pen: walk.pen,
                end_heading: walk.end_heading,
                end_visible: walk.end_visible,
            };

            if dist <= WALK_EPS {
                self.finish_walk(walk);
                return budget;
            }

            let step = budget.min(dist);
            let t = step / dist;
            let nx = self.pose.x + dx * t;
            let ny = self.pose.y + dy * t;
            if let Some((color, width)) = pen {
                self.push_segment(Segment {
                    x0: self.pose.x,
                    y0: self.pose.y,
                    x1: nx,
                    y1: ny,
                    color,
                    width,
                });
            }
            self.pose.x = nx;
            self.pose.y = ny;
            budget -= step;

            if dist - step <= WALK_EPS {
                self.finish_walk(walk);
                return budget;
            }
        }
        0.0
    }

    fn advance_turn(&mut self, mut budget: f64) -> f64 {
        let Some(Motion::Turn { remaining }) = self.motion.as_ref() else {
            return budget;
        };
        let remaining = *remaining;
        if remaining.abs() <= TURN_EPS {
            self.pose.heading = (self.pose.heading + remaining).rem_euclid(360.0);
            self.motion = None;
            return budget;
        }
        if budget <= 0.0 {
            return budget;
        }

        let step = remaining.abs().min(budget * TURN_DEG_PER_PX);
        let signed = remaining.signum() * step;
        self.pose.heading = (self.pose.heading + signed).rem_euclid(360.0);
        let left = remaining - signed;
        budget = (budget - step / TURN_DEG_PER_PX).max(0.0);

        if left.abs() <= TURN_EPS {
            self.pose.heading = (self.pose.heading + left).rem_euclid(360.0);
            self.motion = None;
        } else {
            self.motion = Some(Motion::Turn { remaining: left });
        }
        budget
    }

    fn apply_event_instant(&mut self, event: TurtleEvent) {
        match event {
            TurtleEvent::Line(seg) => {
                self.pose.x = seg.x1;
                self.pose.y = seg.y1;
                self.push_segment(seg);
            }
            TurtleEvent::Move {
                x,
                y,
                heading,
                visible,
            } => {
                self.pose = Pose {
                    x,
                    y,
                    heading,
                    visible,
                };
            }
            TurtleEvent::Turn { delta } => {
                self.pose.heading = (self.pose.heading + delta).rem_euclid(360.0);
            }
            TurtleEvent::Clear => {
                self.segments.clear();
                self.pose = Pose::default();
                self.pen = PenState::default();
            }
            TurtleEvent::Pen { down, color, width } => {
                self.pen = PenState { down, color, width };
            }
            TurtleEvent::Print(text) => match self.current_origin {
                Origin::Script => self.log.push_str(&text),
                Origin::Repl => self.repl.push_output(&text),
            },
            TurtleEvent::Error(text) => match self.current_origin {
                Origin::Script => {
                    self.log.push_str(&text);
                    if !text.ends_with('\n') {
                        self.log.push('\n');
                    }
                }
                Origin::Repl => self.repl.push_error(&text),
            },
            TurtleEvent::Site(span) => {
                if self.current_origin == Origin::Script {
                    self.active_site = span;
                }
            }
            TurtleEvent::Started(origin) => {
                self.current_origin = origin;
            }
            TurtleEvent::Result(value) => {
                self.repl.push_result(&value);
            }
            TurtleEvent::Done => {
                self.in_flight = self.in_flight.saturating_sub(1);
                self.drop_drawing = false;
            }
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

fn is_drawing_event(event: &TurtleEvent) -> bool {
    matches!(
        event,
        TurtleEvent::Line(_)
            | TurtleEvent::Move { .. }
            | TurtleEvent::Turn { .. }
            | TurtleEvent::Pen { .. }
            | TurtleEvent::Site(_)
            | TurtleEvent::Clear
    )
}

/// Byte offset of 0-based `(line, column)` in `text`, where `column` counts chars.
pub fn loc_to_byte(text: &str, line: usize, column: usize) -> Option<usize> {
    let mut offset = 0;
    for (index, row) in text.split('\n').enumerate() {
        if index == line {
            let mut chars = row.char_indices().skip(column);
            return match chars.next() {
                Some((byte, _)) => Some(offset + byte),
                None if row.chars().count() == column => Some(offset + row.len()),
                None => None,
            };
        }
        offset += row.len() + 1;
    }
    None
}

/// Byte range covered by `span` in `text`, if the span still fits the (possibly edited) text.
pub fn span_to_byte_range(text: &str, span: &Span) -> Option<Range<usize>> {
    let start = loc_to_byte(text, span.begin.line, span.begin.column)?;
    let end = loc_to_byte(text, span.end.line, span.end.column)?;
    (start < end).then_some(start..end)
}

/// Status badge text for the canvas corner.
pub fn status_text(pose: &Pose, pen: &PenState) -> String {
    [
        status_row(
            "pos",
            &format!("{} {}", format_num(pose.x), format_num(pose.y)),
        ),
        status_row("heading", &format_num(pose.heading)),
        status_row("turtle", if pose.visible { "show" } else { "hide" }),
        status_row("color", &format_color(pen.color)),
        status_row("size", &format_width(pen.width)),
        status_row("pen?", if pen.down { "down" } else { "up" }),
    ]
    .join("\n")
}

fn status_row(label: &str, value: &str) -> String {
    format!("{label:<7}  {value}")
}

/// A named color such as `blue`, or `#rrggbb` when the channels do not match a name.
pub fn format_color(color: Color) -> String {
    color
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b))
}

fn format_num(n: f64) -> String {
    let rounded = (n * 10.0).round() / 10.0;
    let rounded = if rounded.abs() < 1e-9 { 0.0 } else { rounded };
    format!("{rounded:.1}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub fn format_width(width: f32) -> String {
    let rounded = (width * 100.0).round() / 100.0;
    format!("{rounded:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::Loc;

    fn test_canvas() -> Canvas {
        let mut canvas = Canvas::new();
        canvas.ensure_session(&egui::Context::default());
        canvas
    }

    fn span(b: (usize, usize), e: (usize, usize)) -> Span {
        Span::new(Loc::new(b.0, b.1), Loc::new(e.0, e.1))
    }

    #[test]
    fn apply_event_routes_by_origin() {
        let mut canvas = test_canvas();
        canvas.in_flight = 1;
        canvas.apply_event_instant(TurtleEvent::Started(Origin::Script));
        canvas.apply_event_instant(TurtleEvent::Print("from script\n".into()));
        canvas.apply_event_instant(TurtleEvent::Site(Some(span((0, 0), (0, 3)))));
        canvas.apply_event_instant(TurtleEvent::Done);
        assert_eq!(canvas.log, "from script\n");
        assert!(canvas.repl.transcript.is_empty());
        assert_eq!(canvas.active_site, Some(span((0, 0), (0, 3))));
        assert_eq!(canvas.in_flight, 0);

        canvas.in_flight = 1;
        canvas.active_site = None;
        canvas.apply_event_instant(TurtleEvent::Started(Origin::Repl));
        canvas.apply_event_instant(TurtleEvent::Print("from repl\n".into()));
        canvas.apply_event_instant(TurtleEvent::Site(Some(span((1, 0), (1, 3)))));
        canvas.apply_event_instant(TurtleEvent::Result("42".into()));
        canvas.apply_event_instant(TurtleEvent::Error("boom".into()));
        canvas.apply_event_instant(TurtleEvent::Done);
        assert_eq!(canvas.log, "from script\n");
        assert_eq!(canvas.active_site, None);
        assert_eq!(
            canvas.repl.transcript,
            vec![
                Entry::Output("from repl\n".into()),
                Entry::Result("42".into()),
                Entry::Error("boom\n".into()),
            ]
        );
        assert_eq!(canvas.in_flight, 0);
    }

    #[test]
    fn push_segment_coalesces_collinear_only() {
        let mut canvas = test_canvas();
        canvas.push_segment(Segment {
            x0: 0.0,
            y0: 0.0,
            x1: 0.5,
            y1: 0.0,
            color: Color::BLACK,
            width: 1.5,
        });
        canvas.push_segment(Segment {
            x0: 0.5,
            y0: 0.0,
            x1: 1.0,
            y1: 0.0,
            color: Color::BLACK,
            width: 1.5,
        });
        assert_eq!(canvas.segments.len(), 1);
        assert!((canvas.segments[0].x1 - 1.0).abs() < 1e-9);

        canvas.push_segment(Segment {
            x0: 1.0,
            y0: 0.0,
            x1: 1.0,
            y1: 1.0,
            color: Color::BLACK,
            width: 1.5,
        });
        assert_eq!(canvas.segments.len(), 2);

        canvas.push_segment(Segment {
            x0: 1.0,
            y0: 1.0,
            x1: 2.0,
            y1: 1.0,
            color: Color::RED,
            width: 1.5,
        });
        assert_eq!(canvas.segments.len(), 3);
    }

    #[test]
    fn finish_walk_strokes_remaining_gap() {
        let mut canvas = test_canvas();
        canvas.pose.x = 0.0;
        canvas.pose.y = 0.0;
        canvas.finish_walk(Walk {
            x1: 0.2,
            y1: 0.0,
            pen: Some((Color::BLUE, 2.0)),
            end_heading: None,
            end_visible: None,
        });
        assert_eq!(canvas.segments.len(), 1);
        assert!((canvas.segments[0].x1 - 0.2).abs() < 1e-9);
        assert_eq!(canvas.segments[0].color, Color::BLUE);
        assert!((canvas.pose.x - 0.2).abs() < 1e-9);
    }

    #[test]
    fn request_stop_keeps_control_events() {
        let mut canvas = test_canvas();
        canvas.in_flight = 1;
        canvas.pose.x = 12.0;
        canvas.pose.y = 34.0;
        canvas.pending = vec![
            TurtleEvent::Started(Origin::Repl),
            TurtleEvent::Line(Segment {
                x0: 0.0,
                y0: 0.0,
                x1: 10.0,
                y1: 0.0,
                color: Color::BLACK,
                width: 1.5,
            }),
            TurtleEvent::Result("nil".into()),
            TurtleEvent::Done,
        ];
        canvas.motion = Some(Motion::Turn { remaining: 90.0 });
        canvas.request_stop();
        assert!(canvas.motion.is_none());
        assert!(canvas.drop_drawing);
        assert_eq!(canvas.pending.len(), 3);
        assert!(matches!(canvas.pending[0], TurtleEvent::Started(_)));
        assert!(matches!(canvas.pending[1], TurtleEvent::Result(_)));
        assert!(matches!(canvas.pending[2], TurtleEvent::Done));
        canvas.apply_event_instant(TurtleEvent::Done);
        assert!(!canvas.drop_drawing);
    }

    #[test]
    fn span_to_bytes_single_and_multi_line() {
        let text = "(fd 10)\n(rt 90)\n";
        assert_eq!(span_to_byte_range(text, &span((0, 0), (0, 7))), Some(0..7));
        assert_eq!(span_to_byte_range(text, &span((1, 0), (1, 7))), Some(8..15));
        assert_eq!(span_to_byte_range(text, &span((0, 4), (1, 3))), Some(4..11));
    }

    #[test]
    fn span_to_bytes_counts_chars_not_bytes() {
        let text = "(write \"é\")\n(fd 1)";
        assert_eq!(
            &text[span_to_byte_range(text, &span((0, 7), (0, 10))).unwrap()],
            "\"é\""
        );
        assert_eq!(
            span_to_byte_range(text, &span((1, 0), (1, 6))),
            Some(13..19)
        );
    }

    #[test]
    fn status_text_lists_pose_and_pen() {
        let fresh = status_text(&Pose::default(), &PenState::default());
        assert_eq!(
            fresh,
            "\
pos      0 0
heading  90
turtle   show
color    black
size     1.5
pen?     down"
        );

        let moved = status_text(
            &Pose {
                x: 12.54,
                y: -3.16,
                heading: 0.0,
                visible: false,
            },
            &PenState {
                down: false,
                color: Color::BLUE,
                width: 4.0,
            },
        );
        assert!(moved.contains("pos      12.5 -3.2"));
        assert!(moved.contains("heading  0"));
        assert!(moved.contains("turtle   hide"));
        assert!(moved.contains("color    blue"));
        assert!(moved.contains("pen?     up"));
    }

    #[test]
    fn format_color_uses_name_or_hex() {
        assert_eq!(format_color(Color::BLUE), "blue");
        assert_eq!(
            format_color(Color {
                r: 10,
                g: 20,
                b: 30
            }),
            "#0a141e"
        );
    }

    #[test]
    fn format_width_trims_trailing_zeros() {
        assert_eq!(format_width(1.5), "1.5");
        assert_eq!(format_width(4.0), "4");
        assert_eq!(format_width(0.1), "0.1");
        assert_eq!(format_width(1.25), "1.25");
    }

    #[test]
    fn span_to_bytes_rejects_stale_spans() {
        let text = "(fd 10)";
        assert_eq!(span_to_byte_range(text, &span((0, 0), (0, 8))), None);
        assert_eq!(span_to_byte_range(text, &span((1, 0), (1, 2))), None);
        assert_eq!(span_to_byte_range("", &span((0, 0), (0, 1))), None);
    }
}
