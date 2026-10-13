//! Desktop turtle-graphics host for Rusche.

use std::process::ExitCode;

use rusche_turtle::app::TurtleApp;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let initial = match args.next() {
        Some(arg) if arg == "-h" || arg == "--help" => {
            eprintln!("Usage: rusche-turtle [SCRIPT.rsc]");
            return ExitCode::SUCCESS;
        }
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(src) => Some(src),
            Err(err) => {
                eprintln!("error: failed to read {path}: {err}");
                return ExitCode::from(1);
            }
        },
        None => None,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_title("Rusche Turtle"),
        ..Default::default()
    };

    match eframe::run_native(
        "Rusche Turtle",
        options,
        Box::new(|cc| Ok(Box::new(TurtleApp::new(initial, cc.egui_ctx.clone())))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}
