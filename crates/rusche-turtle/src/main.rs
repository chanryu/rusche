//! Rusche Turtle — a desktop turtle-graphics host for Rusche scripts.

mod app;
mod builtin;
mod runner;
mod turtle;

use std::path::PathBuf;
use std::process::ExitCode;

use app::TurtleApp;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let initial = match args.next() {
        Some(arg) if arg == "-h" || arg == "--help" => {
            eprintln!("Usage: rusche-turtle [SCRIPT.rsc]");
            eprintln!("  Opens a turtle-graphics window. Optional SCRIPT is loaded into the editor.");
            return ExitCode::SUCCESS;
        }
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(src) => Some(src),
            Err(err) => {
                eprintln!("error: failed to read {}: {err}", PathBuf::from(path).display());
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
        Box::new(|_cc| Ok(Box::new(TurtleApp::new(initial)))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}
