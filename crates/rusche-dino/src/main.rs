//! Desktop Chrome-dino-style runner hosted on Rusche.

use std::process::ExitCode;

use rusche_dino::app::DinoApp;
use rusche_dino::sprites::{SCENE_H, SCENE_W, SCALE};

fn main() -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([SCENE_W * SCALE + 40.0, SCENE_H * SCALE + 100.0])
            .with_title("Rusche Dino"),
        ..Default::default()
    };

    match eframe::run_native(
        "Rusche Dino",
        options,
        Box::new(|cc| match DinoApp::new(&cc.egui_ctx) {
            Ok(app) => Ok(Box::new(app)),
            Err(err) => {
                eprintln!("error: failed to load game: {err}");
                Err(err.into())
            }
        }),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}
