//! Canvas painter. A painter foreign is valid only while its canvas procedure runs.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

use rusche::{
    arity_error, get_exact_2_args, ErrorKind, EvalContext, EvalError, EvalResult, Expr, List, NIL,
};

use crate::host::Host;
use crate::value::{self, eval_args};

struct PaintSurface {
    painter: RefCell<egui::Painter>,
    frame: u64,
    active: Cell<bool>,
}

impl PaintSurface {
    fn live(&self, host: &Host) -> bool {
        self.active.get() && self.frame == host.frame()
    }
}

impl rusche::ForeignObject for PaintSurface {
    fn type_name(&self) -> &'static str {
        "painter"
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<painter>")
    }
}

struct Deactivate<'a>(&'a Cell<bool>);

impl Drop for Deactivate<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

pub(crate) fn canvas(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (height_expr, proc_expr) = get_exact_2_args(proc_name, args)?;
    let height = rusche::eval_into_num(proc_name, height_expr, context)?;
    let height = value::require_nonneg(proc_name, "height", height)?;
    let proc = rusche::eval_into_proc(proc_name, proc_expr, context)?;

    let (surface, x, y, w, h) = host.with_ui(proc_name, |ui| {
        let size = egui::vec2(ui.available_width(), height as f32);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        host.set_response(response);
        let surface = Expr::foreign(PaintSurface {
            painter: RefCell::new(painter),
            frame: host.frame(),
            active: Cell::new(false),
        });
        Ok((
            surface,
            rect.min.x as f64,
            rect.min.y as f64,
            rect.width() as f64,
            rect.height() as f64,
        ))
    })?;

    let paint = surface
        .as_foreign::<PaintSurface>()
        .expect("canvas wraps a painter");
    paint.active.set(true);
    let _deactivate = Deactivate(&paint.active);
    context.call(
        &proc,
        &[
            surface,
            Expr::from(x),
            Expr::from(y),
            Expr::from(w),
            Expr::from(h),
        ],
    )
}

fn with_painter<R>(
    host: &Host,
    proc_name: &str,
    value: &Expr,
    f: impl FnOnce(&egui::Painter) -> R,
) -> Result<R, EvalError> {
    let paint = value
        .as_foreign::<PaintSurface>()
        .ok_or_else(|| value::type_error(proc_name, value, "a painter"))?;
    if !paint.live(host) {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: painter is only valid during its canvas callback"),
        ));
    }
    let painter = paint.painter.borrow();
    Ok(f(&painter))
}

fn rect_at(proc_name: &str, x: f64, y: f64, w: f64, h: f64) -> Result<egui::Rect, EvalError> {
    let x = value::require_finite(proc_name, "x", x)?;
    let y = value::require_finite(proc_name, "y", y)?;
    let w = value::require_nonneg(proc_name, "width", w)?;
    let h = value::require_nonneg(proc_name, "height", h)?;
    Ok(egui::Rect::from_min_size(
        egui::pos2(x as f32, y as f32),
        egui::vec2(w as f32, h as f32),
    ))
}

fn stroke_of(color: Option<egui::Color32>, width: f32) -> egui::Stroke {
    match color {
        Some(color) if width > 0.0 => egui::Stroke::new(width, color),
        _ => egui::Stroke::NONE,
    }
}

pub(crate) fn fill_rect(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 6 {
        return Err(arity_error(proc_name, 6..=6, values.len()));
    }
    let x = value::require_num(proc_name, &values[1])?;
    let y = value::require_num(proc_name, &values[2])?;
    let w = value::require_num(proc_name, &values[3])?;
    let h = value::require_num(proc_name, &values[4])?;
    let color = value::require_color(proc_name, &values[5])?;
    let rect = rect_at(proc_name, x, y, w, h)?;
    with_painter(host, proc_name, &values[0], |painter| {
        painter.rect_filled(rect, 0.0, color);
    })?;
    Ok(NIL)
}

pub(crate) fn stroke_rect(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 7 {
        return Err(arity_error(proc_name, 7..=7, values.len()));
    }
    let x = value::require_num(proc_name, &values[1])?;
    let y = value::require_num(proc_name, &values[2])?;
    let w = value::require_num(proc_name, &values[3])?;
    let h = value::require_num(proc_name, &values[4])?;
    let color = value::require_color(proc_name, &values[5])?;
    let width = value::require_nonneg(
        proc_name,
        "width",
        value::require_num(proc_name, &values[6])?,
    )?;
    let rect = rect_at(proc_name, x, y, w, h)?;
    with_painter(host, proc_name, &values[0], |painter| {
        painter.rect_stroke(
            rect,
            0.0,
            stroke_of(Some(color), width as f32),
            egui::StrokeKind::Inside,
        );
    })?;
    Ok(NIL)
}

pub(crate) fn line(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 7 {
        return Err(arity_error(proc_name, 7..=7, values.len()));
    }
    let x0 = value::require_finite(proc_name, "x0", value::require_num(proc_name, &values[1])?)?;
    let y0 = value::require_finite(proc_name, "y0", value::require_num(proc_name, &values[2])?)?;
    let x1 = value::require_finite(proc_name, "x1", value::require_num(proc_name, &values[3])?)?;
    let y1 = value::require_finite(proc_name, "y1", value::require_num(proc_name, &values[4])?)?;
    let color = value::require_color(proc_name, &values[5])?;
    let width = value::require_nonneg(
        proc_name,
        "width",
        value::require_num(proc_name, &values[6])?,
    )?;
    with_painter(host, proc_name, &values[0], |painter| {
        if width > 0.0 {
            painter.line_segment(
                [
                    egui::pos2(x0 as f32, y0 as f32),
                    egui::pos2(x1 as f32, y1 as f32),
                ],
                egui::Stroke::new(width as f32, color),
            );
        }
    })?;
    Ok(NIL)
}

pub(crate) fn polygon(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 5 {
        return Err(arity_error(proc_name, 5..=5, values.len()));
    }
    let points = require_points(proc_name, &values[1])?;
    let fill = value::require_color_or_false(proc_name, &values[2])?;
    let stroke = value::require_color_or_false(proc_name, &values[3])?;
    let width = value::require_nonneg(
        proc_name,
        "width",
        value::require_num(proc_name, &values[4])?,
    )?;
    with_painter(host, proc_name, &values[0], |painter| {
        painter.add(egui::Shape::convex_polygon(
            points,
            fill.unwrap_or(egui::Color32::TRANSPARENT),
            stroke_of(stroke, width as f32),
        ));
    })?;
    Ok(NIL)
}

pub(crate) fn circle(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 7 {
        return Err(arity_error(proc_name, 7..=7, values.len()));
    }
    let x = value::require_finite(proc_name, "x", value::require_num(proc_name, &values[1])?)?;
    let y = value::require_finite(proc_name, "y", value::require_num(proc_name, &values[2])?)?;
    let radius = value::require_nonneg(
        proc_name,
        "radius",
        value::require_num(proc_name, &values[3])?,
    )?;
    let fill = value::require_color_or_false(proc_name, &values[4])?;
    let stroke = value::require_color_or_false(proc_name, &values[5])?;
    let width = value::require_nonneg(
        proc_name,
        "width",
        value::require_num(proc_name, &values[6])?,
    )?;
    with_painter(host, proc_name, &values[0], |painter| {
        painter.circle(
            egui::pos2(x as f32, y as f32),
            radius as f32,
            fill.unwrap_or(egui::Color32::TRANSPARENT),
            stroke_of(stroke, width as f32),
        );
    })?;
    Ok(NIL)
}

pub(crate) fn paint_text(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 6 {
        return Err(arity_error(proc_name, 6..=6, values.len()));
    }
    let x = value::require_finite(proc_name, "x", value::require_num(proc_name, &values[1])?)?;
    let y = value::require_finite(proc_name, "y", value::require_num(proc_name, &values[2])?)?;
    let text = value::require_str(proc_name, &values[3])?.to_string();
    let size = value::require_positive(
        proc_name,
        "size",
        value::require_num(proc_name, &values[4])?,
    )?;
    let color = value::require_color(proc_name, &values[5])?;
    with_painter(host, proc_name, &values[0], |painter| {
        painter.text(
            egui::pos2(x as f32, y as f32),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(size as f32),
            color,
        );
    })?;
    Ok(NIL)
}

fn require_points(proc_name: &str, value: &Expr) -> Result<Vec<egui::Pos2>, EvalError> {
    let Some(list) = value.as_list() else {
        return Err(value::type_error(
            proc_name,
            value,
            "a list of (x y) points",
        ));
    };
    let mut points = Vec::new();
    for item in list.iter() {
        let Some(pair) = item.as_list() else {
            return Err(value::type_error(proc_name, item, "a point (x y)"));
        };
        let mut coords = pair.iter();
        let (Some(x), Some(y), None) = (coords.next(), coords.next(), coords.next()) else {
            return Err(value::type_error(proc_name, item, "a point (x y)"));
        };
        let x = value::require_finite(proc_name, "x", value::require_num(proc_name, x)?)?;
        let y = value::require_finite(proc_name, "y", value::require_num(proc_name, y)?)?;
        points.push(egui::pos2(x as f32, y as f32));
    }
    if points.len() < 3 {
        return Err(value::domain(proc_name, "polygon needs at least 3 points"));
    }
    Ok(points)
}
