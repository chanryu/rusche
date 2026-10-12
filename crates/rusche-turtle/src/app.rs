//! Script editor, Interactive REPL, canvas, and output log.

use std::ops::Range;
use std::sync::mpsc::TryRecvError;

use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, Stroke, Vec2};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, Pos2, TextStyle};
use rusche::Span;

use crate::examples;
use crate::repl::ReplState;
use crate::runner::Session;
use crate::turtle::{Color, Origin, Segment, TurtleEvent, DEFAULT_PEN_WIDTH};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Script,
    Interactive,
}

#[derive(Clone, Copy)]
struct Pose {
    x: f64,
    y: f64,
    heading: f64,
    visible: bool,
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
    Turn { remaining: f64 },
}

/// Pen the canvas is currently showing. Matches a fresh turtle until a script changes it.
struct PenState {
    down: bool,
    color: Color,
    width: f32,
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

struct OpHint {
    form: &'static str,
    blurb: &'static str,
}

const HELP_SECTIONS: &[(&str, &[OpHint])] = &[
    (
        "Motion",
        &[
            OpHint {
                form: "(forward dist)  ; fd",
                blurb: "Move forward (draws if pen down)",
            },
            OpHint {
                form: "(back dist)  ; bk",
                blurb: "Move backward",
            },
            OpHint {
                form: "(left deg)  ; lt",
                blurb: "Turn left",
            },
            OpHint {
                form: "(right deg)  ; rt",
                blurb: "Turn right",
            },
            OpHint {
                form: "(goto x y)",
                blurb: "Absolute move",
            },
            OpHint {
                form: "(setheading deg)  ; seth",
                blurb: "Absolute heading",
            },
            OpHint {
                form: "(home)",
                blurb: "Origin, face north",
            },
        ],
    ),
    (
        "Pen & canvas",
        &[
            OpHint {
                form: "(penup)  ; pu",
                blurb: "Lift pen",
            },
            OpHint {
                form: "(pendown)  ; pd",
                blurb: "Lower pen",
            },
            OpHint {
                form: "(pencolor \"blue\")",
                blurb: "Name or r g b",
            },
            OpHint {
                form: "(pensize w)",
                blurb: "Stroke width",
            },
            OpHint {
                form: "(clear)",
                blurb: "Reset canvas",
            },
            OpHint {
                form: "(hideturtle) / (showturtle)",
                blurb: "Sprite visibility",
            },
        ],
    ),
    (
        "Queries & I/O",
        &[
            OpHint {
                form: "(position)",
                blurb: "→ (x y)",
            },
            OpHint {
                form: "(heading)",
                blurb: "→ degrees (0 east, 90 north)",
            },
            OpHint {
                form: "(pen?)",
                blurb: "→ pen down?",
            },
            OpHint {
                form: "(write …) / (writeln …)",
                blurb: "Output log",
            },
            OpHint {
                form: "(random)",
                blurb: "→ [0, 1)",
            },
            OpHint {
                form: "(repeat n body…)",
                blurb: "Loop body n times",
            },
        ],
    ),
];

pub struct TurtleApp {
    script: String,
    log: String,
    segments: Vec<Segment>,
    pending: Vec<TurtleEvent>,
    motion: Option<Motion>,
    pose: Pose,
    session: Session,
    /// Evaluation requests whose `Done` has not yet been applied.
    in_flight: usize,
    /// Origin of the request currently being drained from `pending`.
    current_origin: Origin,
    tab: Tab,
    repl: ReplState,
    /// Index into [`SPEEDS`], slowest first.
    speed: u32,
    show_help: bool,
    /// Script form whose effects are currently being shown on the canvas.
    active_site: Option<Span>,
    pen: PenState,
}

/// Five animation paces, slowest first. The budget is pixels of travel per frame;
/// turns spend the same budget at [`TURN_DEG_PER_PX`] degrees per pixel.
const SPEEDS: &[(&str, f64)] = &[
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

impl TurtleApp {
    pub fn new(initial_script: Option<String>, ctx: egui::Context) -> Self {
        Self {
            script: initial_script.unwrap_or_else(|| examples::DEFAULT.to_string()),
            log: String::new(),
            segments: Vec::new(),
            pending: Vec::new(),
            motion: None,
            pose: Pose::default(),
            session: Session::spawn(ctx),
            in_flight: 0,
            current_origin: Origin::Script,
            tab: Tab::Script,
            repl: ReplState::default(),
            speed: 2, // Trot, the middle pace
            show_help: false,
            active_site: None,
            pen: PenState::default(),
        }
    }

    fn is_running(&self) -> bool {
        self.in_flight > 0 || !self.pending.is_empty() || self.motion.is_some()
    }

    fn replace_session(&mut self, ctx: &egui::Context) {
        self.session.request_stop();
        self.session = Session::spawn(ctx.clone());
        self.in_flight = 0;
        self.pending.clear();
        self.motion = None;
        self.active_site = None;
    }

    fn start_run(&mut self, ctx: &egui::Context) {
        self.replace_session(ctx);
        self.segments.clear();
        self.pose = Pose::default();
        self.pen = PenState::default();
        self.log.clear();
        self.submit(self.script.clone(), Origin::Script);
    }

    fn reset_session(&mut self, ctx: &egui::Context) {
        self.replace_session(ctx);
        self.segments.clear();
        self.pose = Pose::default();
        self.pen = PenState::default();
        self.repl.clear_transcript();
    }

    fn submit(&mut self, source: String, origin: Origin) {
        if self.session.eval(source, origin) {
            self.in_flight += 1;
        }
    }

    fn request_stop(&mut self) {
        self.session.request_stop();
        self.pending.retain(|e| !is_drawing_event(e));
        self.motion = None;
        self.active_site = None;
    }

    fn clear_canvas(&mut self) {
        self.segments.clear();
        self.pending.retain(|e| !is_drawing_event(e));
        self.motion = None;
        self.pose = Pose::default();
        self.pen = PenState::default();
    }

    fn poll_session(&mut self, ctx: &egui::Context) {
        loop {
            match self.session.events.try_recv() {
                Ok(event) => self.pending.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        if self.is_running() {
            ctx.request_repaint();
        }
    }

    fn apply_pending(&mut self, ctx: &egui::Context) {
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

        if self.motion.is_some() || !self.pending.is_empty() {
            ctx.request_repaint();
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
        // Stroke any leftover gap so short lines and the last Crawl-speed crumb are not lost.
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

    /// Append a stroke, or extend the previous one when it continues the same collinear
    /// pen stroke. Extending keeps Crawl-speed animation as one growing visible segment
    /// instead of many sub-pixel crumbs that round away on screen. A turn starts a new
    /// segment even when color and width match.
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
                // Same direction and collinear (allow for tiny numerical noise).
                if alen > 1e-9
                    && blen > 1e-9
                    && cross.abs() <= 1e-6 * alen * blen
                    && dot > 0.0
                {
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
            }
        }
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Script, "Script");
            ui.selectable_value(&mut self.tab, Tab::Interactive, "Interactive");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.toggle_value(&mut self.show_help, "Help");
            });
        });
    }

    fn editor_controls(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            let running = self.is_running();
            if ui
                .add_enabled(!running, egui::Button::new("Run"))
                .clicked()
            {
                self.start_run(&ctx);
            }
            if ui
                .add_enabled(running, egui::Button::new("Stop"))
                .clicked()
            {
                self.request_stop();
            }
            ui.separator();
            ui.menu_button("Examples", |ui| {
                for example in examples::ALL {
                    if ui.button(example.name).clicked() && !running {
                        self.script = example.source.to_string();
                        ui.close();
                    }
                }
            });
        });
    }

    fn editor_tab(&mut self, ui: &mut egui::Ui) {
        self.editor_controls(ui);
        ui.separator();

        egui::Panel::bottom("log")
            .resizable(true)
            .default_size(160.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Output").strong());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false),
                        );
                    });
            });

        egui::ScrollArea::both().show(ui, |ui| {
            self.script_editor(ui);
        });
    }

    fn interactive_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let running = self.is_running();

        ui.horizontal(|ui| {
            if ui
                .add_enabled(running, egui::Button::new("Stop"))
                .clicked()
            {
                self.request_stop();
            }
            if ui
                .add_enabled(!running, egui::Button::new("Reset"))
                .clicked()
            {
                self.reset_session(&ctx);
            }
        });
        ui.separator();

        if let Some(source) = self.repl.show(ui, running) {
            self.repl.push_input(&source);
            self.submit(source, Origin::Repl);
        }
    }

    fn canvas_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear_canvas();
            }
            ui.separator();
            ui.label("Speed");
            for (i, (label, _)) in SPEEDS.iter().enumerate() {
                ui.selectable_value(&mut self.speed, i as u32, *label);
            }
        });
    }

    fn help_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Turtle API").strong());
        ui.label(
            RichText::new("Origin at center, y up. Heading 0° = east, 90° = north.")
                .small()
                .weak(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Click a form to append it to the script.")
                .small()
                .weak(),
        );
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (section, ops) in HELP_SECTIONS {
                ui.add_space(6.0);
                ui.label(RichText::new(*section).strong());
                for op in *ops {
                    let insert = strip_hint_form(op.form);
                    let response = ui
                        .add(
                            egui::Button::new(
                                RichText::new(op.form).monospace().size(12.0),
                            )
                            .frame(false)
                            .wrap(),
                        )
                        .on_hover_text(format!("Insert `{insert}`"));
                    if response.clicked() {
                        if !self.script.is_empty() && !self.script.ends_with('\n') {
                            self.script.push('\n');
                        }
                        self.script.push_str(insert);
                        self.script.push('\n');
                    }
                    ui.label(RichText::new(op.blurb).small().weak());
                    ui.add_space(2.0);
                }
            }
        });
    }

    /// Code editor that highlights the form whose effects are currently being animated.
    fn script_editor(&mut self, ui: &mut egui::Ui) {
        let active = self.active_site;
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let text = buf.as_str();
            let font_id = TextStyle::Monospace.resolve(ui.style());
            let color = ui
                .visuals()
                .override_text_color
                .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
            let plain = TextFormat {
                font_id,
                color,
                ..Default::default()
            };

            let mut job = LayoutJob::default();
            match active.and_then(|span| span_to_byte_range(text, &span)) {
                Some(range) => {
                    let marked = TextFormat {
                        background: active_site_color(ui.visuals().dark_mode),
                        ..plain.clone()
                    };
                    job.append(&text[..range.start], 0.0, plain.clone());
                    job.append(&text[range.clone()], 0.0, marked);
                    job.append(&text[range.end..], 0.0, plain);
                }
                None => job.append(text, 0.0, plain),
            }
            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut self.script)
                .code_editor()
                .desired_width(f32::INFINITY)
                .desired_rows(30)
                .layouter(&mut layouter),
        );
    }

    fn draw_canvas(&self, ui: &mut egui::Ui, height: f32) {
        let size = egui::vec2(ui.available_width(), height);
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 0.0, Color32::from_rgb(250, 250, 248));

        let origin = rect.center();
        painter.line_segment(
            [pos2(rect.left(), origin.y), pos2(rect.right(), origin.y)],
            Stroke::new(1.0, Color32::from_gray(220)),
        );
        painter.line_segment(
            [pos2(origin.x, rect.top()), pos2(origin.x, rect.bottom())],
            Stroke::new(1.0, Color32::from_gray(220)),
        );

        let to_screen = |x: f64, y: f64| -> Pos2 {
            pos2(origin.x + x as f32, origin.y - y as f32)
        };

        for seg in &self.segments {
            painter.line_segment(
                [to_screen(seg.x0, seg.y0), to_screen(seg.x1, seg.y1)],
                Stroke::new(seg.width, color32(seg.color)),
            );
        }

        if self.pose.visible {
            draw_turtle(
                &painter,
                to_screen(self.pose.x, self.pose.y),
                self.pose.heading,
            );
        }

        draw_status(&painter, rect, &self.pose, &self.pen);
    }
}

/// Readout in the canvas corner: where the turtle is, and the pen it is using.
fn draw_status(painter: &egui::Painter, canvas: egui::Rect, pose: &Pose, pen: &PenState) {
    let text = status_text(pose, pen);
    let lines = text.lines().count().max(1) as f32;
    let color_line = text
        .lines()
        .position(|line| line.starts_with("color"))
        .unwrap_or(0) as f32;
    let galley = painter.layout_no_wrap(
        text,
        egui::FontId::monospace(12.0),
        Color32::from_rgb(40, 40, 40),
    );

    let margin = 8.0;
    let pad = 6.0;
    let swatch = 14.0;
    let gap = 8.0;
    let text_size = galley.size();
    let badge = egui::Rect::from_min_size(
        canvas.min + egui::vec2(margin, margin),
        egui::vec2(
            pad + swatch + gap + text_size.x + pad,
            pad * 2.0 + text_size.y,
        ),
    );
    painter.rect_filled(badge, 4.0, Color32::from_rgba_unmultiplied(255, 255, 255, 230));
    painter.rect_stroke(
        badge,
        4.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        egui::StrokeKind::Inside,
    );

    let text_pos = pos2(badge.left() + pad + swatch + gap, badge.top() + pad);
    let row_h = text_size.y / lines;
    let swatch_rect = egui::Rect::from_min_size(
        pos2(
            badge.left() + pad,
            text_pos.y + row_h * color_line + (row_h - swatch) / 2.0,
        ),
        egui::vec2(swatch, swatch),
    );
    painter.rect_filled(swatch_rect, 2.0, color32(pen.color));
    painter.rect_stroke(
        swatch_rect,
        2.0,
        Stroke::new(1.0, Color32::from_gray(120)),
        egui::StrokeKind::Inside,
    );
    painter.galley(text_pos, galley, Color32::PLACEHOLDER);
}

fn status_text(pose: &Pose, pen: &PenState) -> String {
    [
        status_row("pos", &format!("{} {}", format_num(pose.x), format_num(pose.y))),
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

/// A named color such as `blue`, or `r g b` when the channels do not match a name.
fn format_color(color: Color) -> String {
    color
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{} {} {}", color.r, color.g, color.b))
}

fn format_num(n: f64) -> String {
    let rounded = (n * 10.0).round() / 10.0;
    let rounded = if rounded.abs() < 1e-9 { 0.0 } else { rounded };
    format!("{rounded:.1}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn format_width(width: f32) -> String {
    let rounded = (width * 100.0).round() / 100.0;
    format!("{rounded:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn color32(c: Color) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

fn active_site_color(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(110, 90, 20)
    } else {
        Color32::from_rgb(255, 232, 140)
    }
}

/// Byte offset of 0-based `(line, column)` in `text`, where `column` counts chars.
/// `None` if the location is past the end of its line or the text has fewer lines.
fn loc_to_byte(text: &str, line: usize, column: usize) -> Option<usize> {
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
fn span_to_byte_range(text: &str, span: &Span) -> Option<Range<usize>> {
    let start = loc_to_byte(text, span.begin.line, span.begin.column)?;
    let end = loc_to_byte(text, span.end.line, span.end.column)?;
    (start < end).then_some(start..end)
}

fn strip_hint_form(form: &str) -> &str {
    form.split("  ;")
        .next()
        .unwrap_or(form)
        .split(" / ")
        .next()
        .unwrap_or(form)
        .trim()
}

fn draw_turtle(painter: &egui::Painter, center: Pos2, heading_deg: f64) {
    let rad = heading_deg.to_radians() as f32;
    let forward = Vec2::new(rad.cos(), -rad.sin()); // screen y is down
    let right = Vec2::new(-forward.y, forward.x);
    let at = |along: f32, across: f32| center + forward * along + right * across;

    let shell = Color32::from_rgb(56, 142, 60);
    let shell_dark = Color32::from_rgb(27, 94, 32);
    let skin = Color32::from_rgb(129, 199, 132);
    let skin_dark = Color32::from_rgb(46, 125, 50);
    let outline = Stroke::new(1.0, Color32::from_rgb(27, 54, 30));

    let flipper = |tip: Pos2, a: Pos2, b: Pos2| {
        painter.add(egui::Shape::convex_polygon(vec![tip, a, b], skin, outline));
    };
    flipper(at(6.0, 9.0), at(1.0, 3.0), at(5.0, 2.0));
    flipper(at(6.0, -9.0), at(1.0, -3.0), at(5.0, -2.0));
    flipper(at(-7.0, 8.0), at(-2.0, 3.0), at(-5.0, 2.0));
    flipper(at(-7.0, -8.0), at(-2.0, -3.0), at(-5.0, -2.0));

    painter.add(egui::Shape::convex_polygon(
        vec![at(-11.0, 0.0), at(-6.0, 2.5), at(-6.0, -2.5)],
        skin,
        outline,
    ));
    painter.circle(center, 8.5, shell, outline);
    painter.circle(center, 5.0, shell_dark, Stroke::NONE);
    painter.line_segment([at(0.0, -4.5), at(0.0, 4.5)], Stroke::new(1.0, shell_dark));
    painter.line_segment([at(-4.0, -2.5), at(4.0, 2.5)], Stroke::new(1.0, shell_dark));
    painter.line_segment([at(-4.0, 2.5), at(4.0, -2.5)], Stroke::new(1.0, shell_dark));

    painter.circle(at(11.0, 0.0), 4.0, skin, outline);
    painter.circle(at(12.0, 1.8), 1.1, Color32::WHITE, Stroke::NONE);
    painter.circle(at(12.0, -1.8), 1.1, Color32::WHITE, Stroke::NONE);
    painter.circle(at(12.4, 1.8), 0.55, skin_dark, Stroke::NONE);
    painter.circle(at(12.4, -1.8), 0.55, skin_dark, Stroke::NONE);
}

impl eframe::App for TurtleApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_session(ctx);
        self.apply_pending(ctx);
        if !self.is_running() {
            self.active_site = None;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        egui::Panel::left("side")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| {
                self.tab_strip(ui);
                ui.separator();
                match self.tab {
                    Tab::Script => self.editor_tab(ui),
                    Tab::Interactive => self.interactive_tab(ui),
                }
            });

        if self.show_help {
            egui::Panel::right("help")
                .resizable(true)
                .default_size(280.0)
                .show(ui, |ui| {
                    self.help_panel(ui);
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let controls_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
            let canvas_height = (ui.available_height() - controls_height).max(40.0);
            self.draw_canvas(ui, canvas_height);
            self.canvas_controls(ui);
        });

        if self.is_running() {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self) {
        self.session.request_stop();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::Entry;
    use rusche::Loc;

    fn test_app() -> TurtleApp {
        TurtleApp::new(None, egui::Context::default())
    }

    fn span(b: (usize, usize), e: (usize, usize)) -> Span {
        Span::new(Loc::new(b.0, b.1), Loc::new(e.0, e.1))
    }

    #[test]
    fn apply_event_routes_by_origin() {
        let mut app = test_app();
        app.in_flight = 1;
        app.apply_event_instant(TurtleEvent::Started(Origin::Script));
        app.apply_event_instant(TurtleEvent::Print("from script\n".into()));
        app.apply_event_instant(TurtleEvent::Site(Some(span((0, 0), (0, 3)))));
        app.apply_event_instant(TurtleEvent::Done);
        assert_eq!(app.log, "from script\n");
        assert!(app.repl.transcript.is_empty());
        assert_eq!(app.active_site, Some(span((0, 0), (0, 3))));
        assert_eq!(app.in_flight, 0);

        app.in_flight = 1;
        app.active_site = None;
        app.apply_event_instant(TurtleEvent::Started(Origin::Repl));
        app.apply_event_instant(TurtleEvent::Print("from repl\n".into()));
        app.apply_event_instant(TurtleEvent::Site(Some(span((1, 0), (1, 3)))));
        app.apply_event_instant(TurtleEvent::Result("42".into()));
        app.apply_event_instant(TurtleEvent::Error("boom".into()));
        app.apply_event_instant(TurtleEvent::Done);
        assert_eq!(app.log, "from script\n");
        assert_eq!(app.active_site, None);
        assert_eq!(
            app.repl.transcript,
            vec![
                Entry::Output("from repl\n".into()),
                Entry::Result("42".into()),
                Entry::Error("boom\n".into()),
            ]
        );
        assert_eq!(app.in_flight, 0);
    }

    #[test]
    fn push_segment_coalesces_collinear_only() {
        let mut app = test_app();
        app.push_segment(Segment {
            x0: 0.0,
            y0: 0.0,
            x1: 0.5,
            y1: 0.0,
            color: Color::BLACK,
            width: 1.5,
        });
        app.push_segment(Segment {
            x0: 0.5,
            y0: 0.0,
            x1: 1.0,
            y1: 0.0,
            color: Color::BLACK,
            width: 1.5,
        });
        assert_eq!(app.segments.len(), 1);
        assert!((app.segments[0].x1 - 1.0).abs() < 1e-9);

        // A turn onto a new side must start a fresh segment.
        app.push_segment(Segment {
            x0: 1.0,
            y0: 0.0,
            x1: 1.0,
            y1: 1.0,
            color: Color::BLACK,
            width: 1.5,
        });
        assert_eq!(app.segments.len(), 2);

        app.push_segment(Segment {
            x0: 1.0,
            y0: 1.0,
            x1: 2.0,
            y1: 1.0,
            color: Color::RED,
            width: 1.5,
        });
        assert_eq!(app.segments.len(), 3);
    }

    #[test]
    fn finish_walk_strokes_remaining_gap() {
        let mut app = test_app();
        app.pose.x = 0.0;
        app.pose.y = 0.0;
        app.finish_walk(Walk {
            x1: 0.2,
            y1: 0.0,
            pen: Some((Color::BLUE, 2.0)),
            end_heading: None,
            end_visible: None,
        });
        assert_eq!(app.segments.len(), 1);
        assert!((app.segments[0].x1 - 0.2).abs() < 1e-9);
        assert_eq!(app.segments[0].color, Color::BLUE);
        assert!((app.pose.x - 0.2).abs() < 1e-9);
    }

    #[test]
    fn request_stop_keeps_control_events() {
        let mut app = test_app();
        app.in_flight = 1;
        app.pending = vec![
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
        app.motion = Some(Motion::Turn { remaining: 90.0 });
        app.request_stop();
        assert!(app.motion.is_none());
        assert_eq!(app.pending.len(), 3);
        assert!(matches!(app.pending[0], TurtleEvent::Started(_)));
        assert!(matches!(app.pending[1], TurtleEvent::Result(_)));
        assert!(matches!(app.pending[2], TurtleEvent::Done));
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
        // `é` is two bytes; columns are in chars.
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
    fn format_color_uses_name_or_rgb() {
        assert_eq!(format_color(Color::BLUE), "blue");
        assert_eq!(format_color(Color { r: 10, g: 20, b: 30 }), "10 20 30");
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
