//! Native window. `run-app` blocks in `eframe::run_native`.

use std::rc::Rc;

use rusche::{arity_error, ErrorKind, EvalContext, EvalError, EvalResult, Expr, List, NIL};

use crate::host::Host;
use crate::value::{self, eval_args};

pub(crate) fn run_app(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 4 {
        return Err(arity_error(proc_name, 4..=4, values.len()));
    }
    let title = value::require_str(proc_name, &values[0])?.to_string();
    let width = value::require_positive(
        proc_name,
        "width",
        value::require_num(proc_name, &values[1])?,
    )?;
    let height = value::require_positive(
        proc_name,
        "height",
        value::require_num(proc_name, &values[2])?,
    )?;
    let proc = value::require_proc(proc_name, &values[3])?;
    if host.in_frame() {
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: cannot be called from inside a frame"),
        ));
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (title, width, height, proc);
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: requires a native window"),
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let host = Rc::clone(host);
        let context = context.clone();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([width as f32, height as f32])
                .with_title(title.clone()),
            ..Default::default()
        };
        match eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| {
                Ok(Box::new(EguiApp {
                    host,
                    context,
                    proc,
                    last_error: None,
                }))
            }),
        ) {
            Ok(()) => Ok(NIL),
            Err(err) => Err(EvalError::new(
                ErrorKind::Other,
                format!("{proc_name}: {err}"),
            )),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EguiApp {
    host: Rc<Host>,
    context: EvalContext,
    proc: Expr,
    last_error: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl eframe::App for EguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.host.run_proc(ui, &self.context, &self.proc) {
            Ok(_) => self.last_error = None,
            Err(err) => {
                let message = err.to_string();
                if self.last_error.as_deref() != Some(message.as_str()) {
                    eprintln!("rusche-egui: {message}");
                    self.last_error = Some(message);
                }
            }
        }
    }
}
