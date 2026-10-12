//! Editor, canvas, and output log.

use std::sync::mpsc::TryRecvError;

use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, Stroke, Vec2};
use egui::{pos2, Pos2};

use crate::runner::{self, Job};
use crate::turtle::{Color, Segment, TurtleEvent};

const DEFAULT_SCRIPT: &str = include_str!("../examples/squares.rsc");

struct Example {
    name: &'static str,
    source: &'static str,
}

const EXAMPLES: &[Example] = &[
    Example {
        name: "Nested squares",
        source: include_str!("../examples/squares.rsc"),
    },
    Example {
        name: "Spiral",
        source: include_str!("../examples/spiral.rsc"),
    },
    Example {
        name: "Star",
        source: include_str!("../examples/star.rsc"),
    },
    Example {
        name: "Tree",
        source: include_str!("../examples/tree.rsc"),
    },
    Example {
        name: "Color wheel",
        source: include_str!("../examples/colors.rsc"),
    },
    Example {
        name: "Koch snowflake",
        source: include_str!("../examples/koch.rsc"),
    },
];

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
    job: Option<Job>,
    /// Per-frame travel budget; `0` = instant.
    speed: u32,
    status: String,
    show_help: bool,
}

const WALK_PX_PER_SPEED: f64 = 2.0;
const TURN_DEG_PER_PX: f64 = 2.0;
const WALK_EPS: f64 = 0.25;
const TURN_EPS: f64 = 0.25;

fn face(dx: f64, dy: f64) -> f64 {
    dy.atan2(dx).to_degrees().rem_euclid(360.0)
}

impl TurtleApp {
    pub fn new(initial_script: Option<String>) -> Self {
        Self {
            script: initial_script.unwrap_or_else(|| DEFAULT_SCRIPT.to_string()),
            log: String::new(),
            segments: Vec::new(),
            pending: Vec::new(),
            motion: None,
            pose: Pose::default(),
            job: None,
            speed: 2,
            status: "Ready".into(),
            show_help: true,
        }
    }

    fn is_running(&self) -> bool {
        self.job.is_some() || !self.pending.is_empty() || self.motion.is_some()
    }

    fn start_run(&mut self, ctx: &egui::Context) {
        self.stop_job();
        self.segments.clear();
        self.pending.clear();
        self.motion = None;
        self.pose = Pose::default();
        self.log.clear();
        self.status = "Running…".into();
        self.job = Some(runner::spawn(self.script.clone(), ctx.clone()));
    }

    fn stop_job(&mut self) {
        if let Some(job) = self.job.take() {
            job.request_stop();
            let _ = job.join.join();
        }
    }

    fn request_stop(&mut self) {
        if let Some(job) = &self.job {
            job.request_stop();
            self.status = "Stopping…".into();
        }
        self.pending.clear();
        self.motion = None;
    }

    fn clear_canvas(&mut self) {
        self.segments.clear();
        self.pending.clear();
        self.motion = None;
        self.pose = Pose::default();
    }

    fn poll_job(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.job else {
            return;
        };

        let mut finished = false;
        loop {
            match job.receiver.try_recv() {
                Ok(TurtleEvent::Done) => {
                    finished = true;
                    break;
                }
                Ok(event) => self.pending.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    finished = true;
                    break;
                }
            }
        }

        if finished {
            if let Some(job) = self.job.take() {
                while let Ok(event) = job.receiver.try_recv() {
                    if !matches!(event, TurtleEvent::Done) {
                        self.pending.push(event);
                    }
                }
                let _ = job.join.join();
            }
            if self.status == "Stopping…" {
                self.status = "Stopped".into();
            } else if !self.status.starts_with("error") {
                self.status = "Done".into();
            }
        } else if self.is_running() {
            ctx.request_repaint();
        }
    }

    fn apply_pending(&mut self, ctx: &egui::Context) {
        if self.speed == 0 {
            self.motion = None;
            for event in std::mem::take(&mut self.pending) {
                self.apply_event_instant(event);
            }
            return;
        }

        if self.motion.is_none() && self.pending.is_empty() {
            return;
        }

        let mut budget = self.speed as f64 * WALK_PX_PER_SPEED;
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
                self.segments.push(Segment {
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
                self.segments.push(seg);
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
            }
            TurtleEvent::Print(text) => {
                self.log.push_str(&text);
            }
            TurtleEvent::Error(text) => {
                self.log.push_str(&text);
                if !text.ends_with('\n') {
                    self.log.push('\n');
                }
                self.status = "error".into();
            }
            TurtleEvent::Done => {}
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
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
                for example in EXAMPLES {
                    if ui.button(example.name).clicked() && !running {
                        self.script = example.source.to_string();
                        ui.close();
                    }
                }
            });
            ui.toggle_value(&mut self.show_help, "Help");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(&self.status).weak());
            });
        });
    }

    fn canvas_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear_canvas();
            }
            ui.separator();
            ui.label("Speed");
            ui.add(
                egui::Slider::new(&mut self.speed, 0..=24)
                    .custom_formatter(|n, _| {
                        if n == 0.0 {
                            "instant".into()
                        } else {
                            format!("{:.0} px/frame", n * WALK_PX_PER_SPEED)
                        }
                    })
                    .custom_parser(|s| {
                        if s.eq_ignore_ascii_case("instant") {
                            Some(0.0)
                        } else {
                            let trimmed = s.trim_end_matches("px/frame").trim();
                            trimmed
                                .parse::<f64>()
                                .ok()
                                .map(|px| (px / WALK_PX_PER_SPEED).round())
                        }
                    }),
            );
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
    }
}

fn color32(c: Color) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
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
        self.poll_job(ctx);
        self.apply_pending(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        egui::Panel::top("toolbar").show(ui, |ui| {
            self.toolbar(ui);
        });

        egui::Panel::bottom("log")
            .resizable(true)
            .default_size(120.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Output").strong());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
            });

        egui::Panel::left("editor")
            .resizable(true)
            .default_size(360.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Script").strong());
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.script)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(30),
                    );
                });
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
            ui.label(RichText::new("Canvas").strong());
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
        self.stop_job();
    }
}
