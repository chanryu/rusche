//! Argument parsing shared by the widget and painter natives.

use rusche::{eval, ErrorKind, EvalContext, EvalError, Expr, List};

pub(crate) fn eval_args(args: &List, context: &EvalContext) -> Result<Vec<Expr>, EvalError> {
    args.iter().map(|arg| eval(arg, context)).collect()
}

pub(crate) fn type_error(proc_name: &str, value: &Expr, expected: &str) -> EvalError {
    EvalError::new(
        ErrorKind::Type,
        format!("{proc_name}: expected {expected}, got `{value}`"),
    )
}

pub(crate) fn domain(proc_name: &str, message: impl Into<String>) -> EvalError {
    EvalError::new(ErrorKind::Other, format!("{proc_name}: {}", message.into()))
}

pub(crate) fn require_str<'a>(proc_name: &str, value: &'a Expr) -> Result<&'a str, EvalError> {
    value
        .as_str()
        .ok_or_else(|| type_error(proc_name, value, "a string"))
}

pub(crate) fn require_bool(proc_name: &str, value: &Expr) -> Result<bool, EvalError> {
    value
        .as_bool()
        .ok_or_else(|| type_error(proc_name, value, "a boolean"))
}

pub(crate) fn require_num(proc_name: &str, value: &Expr) -> Result<f64, EvalError> {
    value
        .as_num()
        .ok_or_else(|| type_error(proc_name, value, "a number"))
}

pub(crate) fn require_sym<'a>(proc_name: &str, value: &'a Expr) -> Result<&'a str, EvalError> {
    value
        .as_sym()
        .map(|sym| sym.as_str())
        .ok_or_else(|| type_error(proc_name, value, "a symbol"))
}

pub(crate) fn require_proc(proc_name: &str, value: &Expr) -> Result<Expr, EvalError> {
    if value.as_proc().is_some() {
        Ok(value.clone())
    } else {
        Err(type_error(proc_name, value, "a procedure"))
    }
}

pub(crate) fn require_finite(proc_name: &str, what: &str, value: f64) -> Result<f64, EvalError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(domain(proc_name, format!("{what} must be finite")))
    }
}

pub(crate) fn require_nonneg(proc_name: &str, what: &str, value: f64) -> Result<f64, EvalError> {
    let value = require_finite(proc_name, what, value)?;
    if value >= 0.0 {
        Ok(value)
    } else {
        Err(domain(proc_name, format!("{what} must be >= 0")))
    }
}

pub(crate) fn require_positive(proc_name: &str, what: &str, value: f64) -> Result<f64, EvalError> {
    let value = require_finite(proc_name, what, value)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(domain(proc_name, format!("{what} must be > 0")))
    }
}

/// An integer `>= 1`, such as a text-edit row count.
pub(crate) fn require_count(proc_name: &str, what: &str, value: f64) -> Result<usize, EvalError> {
    if value.is_finite() && value >= 1.0 && value.fract() == 0.0 && value <= usize::MAX as f64 {
        Ok(value as usize)
    } else {
        Err(domain(proc_name, format!("{what} must be an integer >= 1")))
    }
}

/// `(r g b)` with components in `0..255`. Values outside that range are clamped.
pub(crate) fn require_color(proc_name: &str, value: &Expr) -> Result<egui::Color32, EvalError> {
    let Some(list) = value.as_list() else {
        return Err(type_error(proc_name, value, "a color (r g b)"));
    };
    let mut channels = [0u8; 3];
    let mut count = 0;
    for item in list.iter() {
        if count == 3 {
            return Err(type_error(proc_name, value, "a color (r g b)"));
        }
        let n = require_num(proc_name, item)?;
        let n = require_finite(proc_name, "color channel", n)?;
        channels[count] = n.clamp(0.0, 255.0).round() as u8;
        count += 1;
    }
    if count != 3 {
        return Err(type_error(proc_name, value, "a color (r g b)"));
    }
    Ok(egui::Color32::from_rgb(
        channels[0],
        channels[1],
        channels[2],
    ))
}

/// A color, or `false` when the paint operation should skip that part.
pub(crate) fn require_color_or_false(
    proc_name: &str,
    value: &Expr,
) -> Result<Option<egui::Color32>, EvalError> {
    if value.as_bool() == Some(false) {
        Ok(None)
    } else {
        Ok(Some(require_color(proc_name, value)?))
    }
}
