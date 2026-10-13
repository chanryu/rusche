//! Desktop turtle-graphics host for Rusche. The window is a Rusche script on rusche-egui.

mod host;

use std::cell::RefCell;
use std::process::ExitCode;
use std::rc::Rc;

use host::TurtleHost;
use rusche::Evaluator;
use rusche_egui::Egui;
use rusche_turtle_rs::{examples, Canvas};

const UI: &str = include_str!("ui.rsc");

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

    let evaluator = Evaluator::with_prelude();
    let egui = Egui::new();
    let canvas = Rc::new(RefCell::new(Canvas::new()));
    evaluator.install(egui.clone());
    evaluator.install(TurtleHost {
        canvas,
        egui: egui.clone(),
    });

    let script = initial.unwrap_or_else(|| examples::DEFAULT.to_string());
    evaluator
        .root_env()
        .define("initial-script", rusche::Expr::from(script));

    if let Err(err) = evaluator.eval_str(UI) {
        eprintln!("error: {err}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusche::{arity_error, EvalResult, Expr, NIL};

    fn load_ui() -> Result<(), String> {
        let evaluator = Evaluator::with_prelude();
        let egui = Egui::new();
        let canvas = Rc::new(RefCell::new(Canvas::new()));
        evaluator.install(egui.clone());
        evaluator.install(TurtleHost {
            canvas,
            egui: egui.clone(),
        });
        evaluator
            .root_env()
            .define("initial-script", Expr::from(examples::DEFAULT));
        // Skip the native window; still evaluate every definition.
        evaluator
            .root_env()
            .define_native_proc("run-app", |proc_name, args, _| -> EvalResult {
                if args.len() != 4 {
                    return Err(arity_error(proc_name, 4..=4, args.len()));
                }
                Ok(NIL)
            });
        evaluator
            .eval_str(UI)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn ui_script_loads() {
        load_ui().unwrap_or_else(|err| panic!("{err}"));
    }

    #[test]
    fn headless_frame_builds_chrome() {
        let evaluator = Evaluator::with_prelude();
        let egui = Egui::new();
        let canvas = Rc::new(RefCell::new(Canvas::new()));
        evaluator.install(egui.clone());
        evaluator.install(TurtleHost {
            canvas: Rc::clone(&canvas),
            egui: egui.clone(),
        });
        evaluator
            .root_env()
            .define("initial-script", Expr::from("; empty\n"));
        evaluator
            .root_env()
            .define_native_proc("run-app", |proc_name, args, _| -> EvalResult {
                if args.len() != 4 {
                    return Err(arity_error(proc_name, 4..=4, args.len()));
                }
                Ok(NIL)
            });
        evaluator.eval_str(UI).unwrap();
        let ui = evaluator.eval_str("ui").unwrap();
        egui.run_frame(evaluator.context(), &ui, 1100.0, 720.0)
            .unwrap_or_else(|err| panic!("{err}"));
    }
}
