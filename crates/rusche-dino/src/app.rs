//! Desktop window: call Rusche `tick` each frame and paint the scene list.

use eframe::egui::{self, Key, RichText};
use rusche::{Evaluator, Expr};

use crate::load_game;
use crate::scene;
use crate::sprites::{SpriteAtlas, SCENE_H, SCENE_W, SCALE};

pub struct DinoApp {
    evaluator: Evaluator,
    tick: Expr,
    atlas: SpriteAtlas,
    last_scene: Expr,
    error: Option<String>,
    stopped: bool,
}

impl DinoApp {
    pub fn new(ctx: &egui::Context) -> Result<Self, String> {
        let (evaluator, tick) = load_game().map_err(|e| e.to_string())?;
        Ok(Self {
            evaluator,
            tick,
            atlas: SpriteAtlas::load(ctx),
            last_scene: rusche::NIL,
            error: None,
            stopped: false,
        })
    }

    fn frame(&mut self, dt: f64, jump: bool, duck: bool) {
        if self.stopped {
            return;
        }
        match self.evaluator.call(
            &self.tick,
            &[Expr::from(dt), Expr::from(jump), Expr::from(duck)],
        ) {
            Ok(scene) => self.last_scene = scene,
            Err(err) => {
                self.error = Some(err.to_string());
                self.stopped = true;
            }
        }
    }
}

impl eframe::App for DinoApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.unstable_dt).min(0.05) as f64;
        let jump = ctx.input(|i| {
            i.key_down(Key::Space) || i.key_down(Key::ArrowUp) || i.pointer.any_down()
        });
        let duck = ctx.input(|i| i.key_down(Key::ArrowDown));
        self.frame(dt, jump, duck);
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Rusche Dino").strong().size(18.0));
                ui.label("Space / Up / click: jump | Down: duck | jump to restart");
                ui.add_space(8.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, format!("error: {err}"));
                }

                let result = scene::paint(ui, &self.atlas, &self.last_scene);
                if let Err(err) = result {
                    if self.error.is_none() {
                        self.error = Some(err.to_string());
                        self.stopped = true;
                    }
                    ui.colored_label(egui::Color32::RED, format!("scene: {err}"));
                }

                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "{}×{} @ {}×",
                        SCENE_W as i32,
                        SCENE_H as i32,
                        SCALE as i32
                    ))
                    .small()
                    .weak(),
                );
            });
        });
    }
}
