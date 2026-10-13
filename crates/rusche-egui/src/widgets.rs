//! Panels, widgets, scroll areas, and key queries.

use std::cell::RefCell;
use std::rc::Rc;

use egui::text::{LayoutJob, TextFormat};
use rusche::{
    arity_error, get_exact_1_arg, ErrorKind, EvalContext, EvalError, EvalResult, Expr, List, NIL,
};

use crate::buffer::{self, Buffer};
use crate::host::{self, Host};
use crate::value::{self, eval_args};

enum Side {
    Left,
    Right,
    Bottom,
}

pub(crate) fn left_panel(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    side_panel(host, proc_name, args, context, Side::Left)
}

pub(crate) fn right_panel(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    side_panel(host, proc_name, args, context, Side::Right)
}

pub(crate) fn bottom_panel(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    side_panel(host, proc_name, args, context, Side::Bottom)
}

fn side_panel(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
    side: Side,
) -> EvalResult {
    let values = eval_args(args, context)?;
    let (id, size, resizable, proc) = match values.len() {
        3 => (
            value::require_str(proc_name, &values[0])?.to_string(),
            value::require_num(proc_name, &values[1])?,
            true,
            value::require_proc(proc_name, &values[2])?,
        ),
        4 => (
            value::require_str(proc_name, &values[0])?.to_string(),
            value::require_num(proc_name, &values[1])?,
            value::require_bool(proc_name, &values[2])?,
            value::require_proc(proc_name, &values[3])?,
        ),
        n => return Err(arity_error(proc_name, 3..=4, n)),
    };
    let size = value::require_nonneg(proc_name, "size", size)?;
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        let panel = match side {
            Side::Left => egui::Panel::left(id),
            Side::Right => egui::Panel::right(id),
            Side::Bottom => egui::Panel::bottom(id),
        };
        panel
            .resizable(resizable)
            .default_size(size as f32)
            .show(ui, |ui| {
                result = host.enter(ui, || context.call(&proc, &[]));
            });
        result
    })
}

pub(crate) fn central_panel(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let proc = one_proc(proc_name, args, context)?;
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        egui::CentralPanel::default().show(ui, |ui| {
            result = host.enter(ui, || context.call(&proc, &[]));
        });
        result
    })
}

pub(crate) fn horizontal(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let proc = one_proc(proc_name, args, context)?;
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        ui.horizontal(|ui| {
            result = host.enter(ui, || context.call(&proc, &[]));
        });
        result
    })
}

pub(crate) fn right_to_left(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let proc = one_proc(proc_name, args, context)?;
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            result = host.enter(ui, || context.call(&proc, &[]));
        });
        result
    })
}

pub(crate) fn scroll(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    let (direction, stick, proc) = match values.len() {
        2 => (
            value::require_sym(proc_name, &values[0])?,
            false,
            value::require_proc(proc_name, &values[1])?,
        ),
        3 => {
            let direction = value::require_sym(proc_name, &values[0])?;
            let flag = value::require_sym(proc_name, &values[1])?;
            if flag != "stick-to-bottom" {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: expected 'stick-to-bottom, got `{flag}`"),
                ));
            }
            (direction, true, value::require_proc(proc_name, &values[2])?)
        }
        n => return Err(arity_error(proc_name, 2..=3, n)),
    };
    let vertical = match direction {
        "vertical" => true,
        "both" => false,
        other => {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: expected 'vertical or 'both, got `{other}`"),
            ));
        }
    };
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        let area = if vertical {
            egui::ScrollArea::vertical()
        } else {
            egui::ScrollArea::both()
        };
        area.stick_to_bottom(stick).show(ui, |ui| {
            result = host.enter(ui, || context.call(&proc, &[]));
        });
        result
    })
}

pub(crate) fn separator(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    host.with_ui(proc_name, |ui| {
        let response = ui.separator();
        host.set_response(response);
        Ok(NIL)
    })
}

pub(crate) fn space(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let amount = rusche::eval_into_num(proc_name, arg, context)?;
    let amount = value::require_nonneg(proc_name, "amount", amount)?;
    host.with_ui(proc_name, |ui| {
        ui.add_space(amount as f32);
        Ok(NIL)
    })
}

pub(crate) fn available_width(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    measure(host, proc_name, args, |ui| ui.available_width() as f64)
}

pub(crate) fn available_height(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    measure(host, proc_name, args, |ui| ui.available_height() as f64)
}

pub(crate) fn interact_height(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    measure(host, proc_name, args, |ui| {
        let spacing = ui.spacing();
        (spacing.interact_size.y + spacing.item_spacing.y) as f64
    })
}

fn measure(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    f: impl FnOnce(&egui::Ui) -> f64,
) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    let value = host.with_ui(proc_name, |ui| Ok(f(ui)))?;
    Ok(Expr::from(value))
}

struct LabelStyle {
    strong: bool,
    small: bool,
    weak: bool,
    monospace: bool,
    italics: bool,
    color: Option<egui::Color32>,
}

pub(crate) fn label(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.is_empty() {
        return Err(arity_error(proc_name, 1..=1, 0));
    }
    let text = value::require_str(proc_name, &values[0])?.to_string();
    let mut style = LabelStyle {
        strong: false,
        small: false,
        weak: false,
        monospace: false,
        italics: false,
        color: None,
    };
    for arg in &values[1..] {
        if let Some(name) = arg.as_sym().map(|sym| sym.as_str()) {
            match name {
                "strong" => style.strong = true,
                "small" => style.small = true,
                "weak" => style.weak = true,
                "monospace" => style.monospace = true,
                "italics" => style.italics = true,
                other => {
                    return Err(EvalError::new(
                        ErrorKind::Type,
                        format!("{proc_name}: unknown style `{other}`"),
                    ));
                }
            }
        } else if style.color.is_some() {
            return Err(value::type_error(proc_name, arg, "one color"));
        } else {
            style.color = Some(value::require_color(proc_name, arg)?);
        }
    }
    host.with_ui(proc_name, |ui| {
        let mut rich = egui::RichText::new(text);
        if style.strong {
            rich = rich.strong();
        }
        if style.small {
            rich = rich.small();
        }
        if style.weak {
            rich = rich.weak();
        }
        if style.monospace {
            rich = rich.monospace();
        }
        if style.italics {
            rich = rich.italics();
        }
        if let Some(color) = style.color {
            rich = rich.color(color);
        }
        let response = ui.label(rich);
        host.set_response(response);
        Ok(NIL)
    })
}

pub(crate) fn button(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    let (text, enabled) = match values.len() {
        1 => (value::require_str(proc_name, &values[0])?.to_string(), true),
        2 => {
            let text = value::require_str(proc_name, &values[0])?.to_string();
            let flag = value::require_sym(proc_name, &values[1])?;
            if flag != "disabled" {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: expected 'disabled, got `{flag}`"),
                ));
            }
            (text, false)
        }
        n => return Err(arity_error(proc_name, 1..=2, n)),
    };
    let clicked = host.with_ui(proc_name, |ui| {
        let response = ui.add_enabled(enabled, egui::Button::new(text));
        let clicked = response.clicked();
        host.set_response(response);
        Ok(clicked)
    })?;
    Ok(Expr::from(clicked))
}

pub(crate) fn selectable(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (selected, text) = selected_label(proc_name, args, context)?;
    let selected = host.with_ui(proc_name, |ui| {
        let response = ui.selectable_label(selected, text);
        let selected = response.clicked() || selected;
        host.set_response(response);
        Ok(selected)
    })?;
    Ok(Expr::from(selected))
}

pub(crate) fn toggle(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let (selected, text) = selected_label(proc_name, args, context)?;
    let selected = host.with_ui(proc_name, |ui| {
        let mut selected = selected;
        let response = ui.toggle_value(&mut selected, text);
        host.set_response(response);
        Ok(selected)
    })?;
    Ok(Expr::from(selected))
}

fn selected_label(
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> Result<(bool, String), EvalError> {
    let values = eval_args(args, context)?;
    if values.len() != 2 {
        return Err(arity_error(proc_name, 2..=2, values.len()));
    }
    let selected = value::require_bool(proc_name, &values[0])?;
    let text = value::require_str(proc_name, &values[1])?.to_string();
    Ok((selected, text))
}

pub(crate) fn menu(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.len() != 2 {
        return Err(arity_error(proc_name, 2..=2, values.len()));
    }
    let title = value::require_str(proc_name, &values[0])?.to_string();
    let proc = value::require_proc(proc_name, &values[1])?;
    host.with_ui(proc_name, |ui| {
        let mut result = Ok(NIL);
        let response = ui.menu_button(title, |ui| {
            result = host.enter(ui, || context.call(&proc, &[]));
        });
        host.set_response(response.response);
        result
    })
}

pub(crate) fn close_menu(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    host.with_ui(proc_name, |ui| {
        ui.close();
        Ok(NIL)
    })
}

pub(crate) fn hover(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let text = rusche::eval_into_str(proc_name, arg, context)?;
    if !host.in_frame() {
        return Err(host::outside(proc_name));
    }
    let Some(response) = host.take_response() else {
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: no widget to attach hover text to"),
        ));
    };
    host.set_response(response.on_hover_text(text));
    Ok(NIL)
}

struct TextOpts {
    code: bool,
    read_only: bool,
    submit_on_enter: bool,
    focus: bool,
    rows: Option<usize>,
    layouter: Option<Expr>,
}

pub(crate) fn text_edit(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let values = eval_args(args, context)?;
    if values.is_empty() {
        return Err(arity_error(proc_name, 1..=1, 0));
    }
    let buffer = buffer::require_buffer(proc_name, &values[0])?;
    let opts = text_opts(proc_name, &values[1..])?;
    let focused = host.with_ui(proc_name, |ui| {
        show_text_edit(host, ui, context, &buffer, opts)
    })?;
    Ok(Expr::from(focused))
}

fn text_opts(proc_name: &str, args: &[Expr]) -> Result<TextOpts, EvalError> {
    let mut opts = TextOpts {
        code: false,
        read_only: false,
        submit_on_enter: false,
        focus: false,
        rows: None,
        layouter: None,
    };
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg.as_proc().is_some() {
            if opts.layouter.is_some() {
                return Err(value::domain(proc_name, "text-edit takes one layouter"));
            }
            opts.layouter = Some(arg.clone());
            index += 1;
            continue;
        }
        let name = value::require_sym(proc_name, arg)?;
        match name {
            "code" => opts.code = true,
            "read-only" => opts.read_only = true,
            "submit-on-enter" => opts.submit_on_enter = true,
            "focus" => opts.focus = true,
            "rows" => {
                index += 1;
                let Some(next) = args.get(index) else {
                    return Err(value::domain(proc_name, "'rows requires a number"));
                };
                let rows =
                    value::require_count(proc_name, "rows", value::require_num(proc_name, next)?)?;
                opts.rows = Some(rows);
            }
            other => {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: unknown option `{other}`"),
                ));
            }
        }
        index += 1;
    }
    Ok(opts)
}

fn show_text_edit(
    host: &Host,
    ui: &mut egui::Ui,
    context: &EvalContext,
    buffer: &Buffer,
    opts: TextOpts,
) -> Result<bool, EvalError> {
    let move_end = buffer.take_cursor_end();
    let mut held = buffer.take_text();
    let layouter_error: Rc<RefCell<Option<EvalError>>> = Rc::new(RefCell::new(None));
    let code = opts.code;

    let mut output = {
        let mut edit = egui::TextEdit::multiline(&mut held.text).desired_width(f32::INFINITY);
        if opts.code {
            edit = edit.code_editor();
        }
        if opts.read_only {
            edit = edit.interactive(false);
        }
        if let Some(rows) = opts.rows {
            edit = edit.desired_rows(rows);
        }
        if opts.submit_on_enter {
            edit = edit.return_key(Some(egui::KeyboardShortcut::new(
                egui::Modifiers::SHIFT,
                egui::Key::Enter,
            )));
        }
        if let Some(proc) = opts.layouter.clone() {
            let error_slot = Rc::clone(&layouter_error);
            let eval_ctx = context.clone();
            let mut layouter = move |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let text = buf.as_str();
                let job = match eval_ctx.call(&proc, &[Expr::from(text)]) {
                    Ok(value) => match spans_to_job(ui, text, &value, code) {
                        Ok(job) => job,
                        Err(err) => {
                            remember_error(&error_slot, err);
                            plain_job(ui, text, wrap_width, code)
                        }
                    },
                    Err(err) => {
                        remember_error(&error_slot, err);
                        plain_job(ui, text, wrap_width, code)
                    }
                };
                let mut job = job;
                job.wrap.max_width = wrap_width;
                ui.fonts_mut(|fonts| fonts.layout_job(job))
            };
            edit.layouter(&mut layouter).show(ui)
        } else {
            edit.show(ui)
        }
    };

    if move_end {
        let end = egui::text::CCursor::new(held.text.chars().count());
        output
            .state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(end)));
        output.state.store(ui.ctx(), output.response.id);
    }
    if opts.focus {
        output.response.request_focus();
    }
    let focused = output.response.has_focus();
    host.set_response(output.response.response.clone());
    // Layouter already fell back to plain text; do not abort the frame (and
    // skip later panels) just because syntax highlighting failed.
    if let Some(err) = layouter_error.borrow_mut().take() {
        eprintln!("rusche-egui: text-edit layouter error: {err}");
    }
    Ok(focused)
}

fn remember_error(slot: &RefCell<Option<EvalError>>, err: EvalError) {
    let mut slot = slot.borrow_mut();
    if slot.is_none() {
        *slot = Some(err);
    }
}

fn plain_job(ui: &egui::Ui, text: &str, wrap_width: f32, code: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(text, 0.0, text_format(ui, code, default_color(ui), None));
    job.wrap.max_width = wrap_width;
    job
}

fn spans_to_job(
    ui: &egui::Ui,
    text: &str,
    value: &Expr,
    code: bool,
) -> Result<LayoutJob, EvalError> {
    let proc_name = "text-edit";
    let Some(list) = value.as_list() else {
        return Err(value::type_error(
            proc_name,
            value,
            "a list of (start end r g b) spans",
        ));
    };
    let mut spans = Vec::new();
    for item in list.iter() {
        spans.push(parse_span(proc_name, text, item)?);
    }
    spans.sort_by_key(|span| span.start);
    let color = default_color(ui);
    let mut job = LayoutJob::default();
    let mut cursor = 0;
    for span in spans {
        if span.start < cursor {
            return Err(value::domain(proc_name, "overlapping layouter span"));
        }
        if cursor < span.start {
            job.append(
                &text[cursor..span.start],
                0.0,
                text_format(ui, code, color, None),
            );
        }
        job.append(
            &text[span.start..span.end],
            0.0,
            text_format(ui, code, span.color, span.background),
        );
        cursor = span.end;
    }
    if cursor < text.len() {
        job.append(&text[cursor..], 0.0, text_format(ui, code, color, None));
    }
    Ok(job)
}

struct ByteSpan {
    start: usize,
    end: usize,
    color: egui::Color32,
    background: Option<egui::Color32>,
}

fn parse_span(proc_name: &str, text: &str, value: &Expr) -> Result<ByteSpan, EvalError> {
    let Some(list) = value.as_list() else {
        return Err(value::type_error(
            proc_name,
            value,
            "a span (start end r g b) or (start end r g b br bg bb)",
        ));
    };
    let nums: Vec<&Expr> = list.iter().collect();
    if nums.len() != 5 && nums.len() != 8 {
        return Err(value::type_error(
            proc_name,
            value,
            "a span (start end r g b) or (start end r g b br bg bb)",
        ));
    }
    let start = byte_index(proc_name, text, value::require_num(proc_name, nums[0])?)?;
    let end = byte_index(proc_name, text, value::require_num(proc_name, nums[1])?)?;
    if start > end {
        return Err(value::domain(proc_name, "span start is past its end"));
    }
    let color = value::require_color(
        proc_name,
        &Expr::from(vec![nums[2].clone(), nums[3].clone(), nums[4].clone()]),
    )?;
    let background = if nums.len() == 8 {
        Some(value::require_color(
            proc_name,
            &Expr::from(vec![nums[5].clone(), nums[6].clone(), nums[7].clone()]),
        )?)
    } else {
        None
    };
    Ok(ByteSpan {
        start,
        end,
        color,
        background,
    })
}

fn byte_index(proc_name: &str, text: &str, value: f64) -> Result<usize, EvalError> {
    if !(value.is_finite() && value >= 0.0 && value.fract() == 0.0) {
        return Err(value::domain(
            proc_name,
            "span offsets must be non-negative integers",
        ));
    }
    let index = value as usize;
    if index > text.len() || !text.is_char_boundary(index) {
        return Err(value::domain(
            proc_name,
            format!("span offset {index} is outside the text"),
        ));
    }
    Ok(index)
}

fn default_color(ui: &egui::Ui) -> egui::Color32 {
    ui.visuals()
        .override_text_color
        .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color())
}

fn text_format(
    ui: &egui::Ui,
    code: bool,
    color: egui::Color32,
    background: Option<egui::Color32>,
) -> TextFormat {
    let font_id = if code {
        egui::TextStyle::Monospace
    } else {
        egui::TextStyle::Body
    }
    .resolve(ui.style());
    TextFormat {
        font_id,
        color,
        background: background.unwrap_or(egui::Color32::TRANSPARENT),
        ..Default::default()
    }
}

pub(crate) fn key_pressed(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let value = rusche::eval(arg, context)?;
    let name = value::require_sym(proc_name, &value)?;
    let key = match name {
        "enter" => egui::Key::Enter,
        "up" => egui::Key::ArrowUp,
        "down" => egui::Key::ArrowDown,
        other => {
            return Err(EvalError::new(
                ErrorKind::Type,
                format!("{proc_name}: expected 'enter, 'up, or 'down, got `{other}`"),
            ));
        }
    };
    let pressed = host.with_ui(proc_name, |ui| Ok(ui.input(|input| input.key_pressed(key))))?;
    Ok(Expr::from(pressed))
}

pub(crate) fn shift_down(
    host: &Rc<Host>,
    proc_name: &str,
    args: &List,
    _context: &EvalContext,
) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    let down = host.with_ui(proc_name, |ui| Ok(ui.input(|input| input.modifiers.shift)))?;
    Ok(Expr::from(down))
}

fn one_proc(proc_name: &str, args: &List, context: &EvalContext) -> Result<Expr, EvalError> {
    let arg = get_exact_1_arg(proc_name, args)?;
    rusche::eval_into_proc(proc_name, arg, context)
}
