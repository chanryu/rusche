//! Script editor, REPL, canvas, and output log.

use std::ops::Range;

use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, Stroke, Vec2};
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, Pos2, TextStyle};

use crate::canvas::{span_to_byte_range, status_text, Canvas, PenState, Pose, SPEEDS};
use crate::examples;
use crate::highlight;
use crate::turtle::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Script,
    Repl,
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
    canvas: Canvas,
    tab: Tab,
    show_help: bool,
}

impl TurtleApp {
    pub fn new(initial_script: Option<String>, ctx: egui::Context) -> Self {
        let mut canvas = Canvas::new();
        canvas.ensure_session(&ctx);
        Self {
            script: initial_script.unwrap_or_else(|| examples::DEFAULT.to_string()),
            canvas,
            tab: Tab::Script,
            show_help: false,
        }
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Script, "Script");
            ui.selectable_value(&mut self.tab, Tab::Repl, "REPL");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.toggle_value(&mut self.show_help, "Help");
            });
        });
    }

    fn editor_controls(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            let running = self.canvas.is_running();
            if ui.add_enabled(!running, egui::Button::new("Run")).clicked() {
                self.canvas.run_script(self.script.clone());
            }
            if ui.add_enabled(running, egui::Button::new("Stop")).clicked() {
                self.canvas.request_stop();
            }
            if ui
                .add_enabled(!running, egui::Button::new("Reset"))
                .clicked()
            {
                self.canvas.reset(&ctx);
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
                            egui::TextEdit::multiline(&mut self.canvas.log)
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

    fn repl_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let running = self.canvas.is_running();

        ui.horizontal(|ui| {
            if ui.add_enabled(running, egui::Button::new("Stop")).clicked() {
                self.canvas.request_stop();
            }
            if ui
                .add_enabled(!running, egui::Button::new("Reset"))
                .clicked()
            {
                self.canvas.reset(&ctx);
            }
        });
        ui.separator();

        if let Some(source) = self.canvas.repl.show(ui, running) {
            self.canvas.repl.push_input(&source);
            self.canvas.run_repl(source);
        }
    }

    fn canvas_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.canvas.clear();
            }
            ui.separator();
            ui.label(RichText::new("Speed").small().weak());
            for (i, (label, _)) in SPEEDS.iter().enumerate() {
                ui.selectable_value(&mut self.canvas.speed, i as u32, *label);
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
                            egui::Button::new(RichText::new(op.form).monospace().size(12.0))
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

    /// Code editor with syntax coloring; also marks the form currently being animated.
    fn script_editor(&mut self, ui: &mut egui::Ui) {
        let active = self.canvas.active_site;
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let text = buf.as_str();
            let font_id = TextStyle::Monospace.resolve(ui.style());
            let default_color = ui
                .visuals()
                .override_text_color
                .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
            let dark = ui.visuals().dark_mode;
            let mark_bg = active_site_color(dark);
            let active_range = active.and_then(|span| span_to_byte_range(text, &span));

            let mut job = LayoutJob::default();
            for span in highlight::classify(text) {
                let color = highlight::color_for(span.kind, dark, default_color);
                append_syntax_span(
                    &mut job,
                    text,
                    span.range,
                    active_range.as_ref(),
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                    mark_bg,
                );
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

        let to_screen = |x: f64, y: f64| -> Pos2 { pos2(origin.x + x as f32, origin.y - y as f32) };

        for seg in &self.canvas.segments {
            painter.line_segment(
                [to_screen(seg.x0, seg.y0), to_screen(seg.x1, seg.y1)],
                Stroke::new(seg.width, color32(seg.color)),
            );
        }

        if self.canvas.pose.visible {
            draw_turtle(
                &painter,
                to_screen(self.canvas.pose.x, self.canvas.pose.y),
                self.canvas.pose.heading,
            );
        }

        draw_status(&painter, rect, &self.canvas.pose, &self.canvas.pen);
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
    painter.rect_filled(
        badge,
        4.0,
        Color32::from_rgba_unmultiplied(255, 255, 255, 230),
    );
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

/// Append `range` of `text` to `job`, splitting so the active-site background applies only
/// where it overlaps.
fn append_syntax_span(
    job: &mut LayoutJob,
    text: &str,
    range: Range<usize>,
    active: Option<&Range<usize>>,
    base: TextFormat,
    mark_bg: Color32,
) {
    let Some(active) = active else {
        job.append(&text[range], 0.0, base);
        return;
    };
    let start = range.start;
    let end = range.end;
    let mid_start = start.max(active.start);
    let mid_end = end.min(active.end);

    if mid_start >= mid_end {
        job.append(&text[start..end], 0.0, base);
        return;
    }

    if start < mid_start {
        job.append(&text[start..mid_start], 0.0, base.clone());
    }
    let mut marked = base.clone();
    marked.background = mark_bg;
    job.append(&text[mid_start..mid_end], 0.0, marked);
    if mid_end < end {
        job.append(&text[mid_end..end], 0.0, base);
    }
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
        self.canvas.frame(ctx);
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
                    Tab::Repl => self.repl_tab(ui),
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

        if self.canvas.is_running() {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self) {
        self.canvas.shutdown();
    }
}
