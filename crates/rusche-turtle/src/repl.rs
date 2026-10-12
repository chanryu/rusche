//! REPL state: transcript, history, and input completeness.

use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
use egui::{self, Key, Modifiers, RichText, TextEdit, TextStyle};

use rusche::{tokenize, LexError, ParseError, Parser};

use crate::highlight;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Input(String),
    Output(String),
    Result(String),
    Error(String),
}

#[derive(Default)]
pub struct ReplState {
    pub input: String,
    pub transcript: Vec<Entry>,
    history: Vec<String>,
    history_pos: Option<usize>,
    request_focus: bool,
    /// After history navigation, place the caret at the end of the loaded line.
    cursor_to_end: bool,
}

impl ReplState {
    pub fn push_input(&mut self, text: &str) {
        self.transcript.push(Entry::Input(text.to_string()));
    }

    pub fn push_output(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.transcript.push(Entry::Output(text.to_string()));
    }

    pub fn push_result(&mut self, value: &str) {
        self.transcript.push(Entry::Result(value.to_string()));
    }

    pub fn push_error(&mut self, text: &str) {
        let mut text = text.to_string();
        if !text.ends_with('\n') {
            text.push('\n');
        }
        self.transcript.push(Entry::Error(text));
    }

    pub fn clear_transcript(&mut self) {
        self.transcript.clear();
    }

    /// Draw the transcript and input box. Returns submitted source when Enter evaluates.
    pub fn show(&mut self, ui: &mut egui::Ui, busy: bool) -> Option<String> {
        // Reserve the input strip first so the transcript ScrollArea cannot eat it.
        let mut submitted = None;
        egui::Panel::bottom("repl_input")
            .resizable(true)
            .default_size(100.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Enter evaluates, Shift+Enter inserts a newline.")
                        .small()
                        .weak(),
                );

                let shift_enter = egui::KeyboardShortcut::new(Modifiers::SHIFT, Key::Enter);
                let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                    let text = buf.as_str();
                    let font_id = TextStyle::Monospace.resolve(ui.style());
                    let default_color = ui
                        .visuals()
                        .override_text_color
                        .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
                    let dark = ui.visuals().dark_mode;

                    let mut job = LayoutJob::default();
                    for span in highlight::classify(text) {
                        job.append(
                            &text[span.range.clone()],
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color: highlight::color_for(span.kind, dark, default_color),
                                ..Default::default()
                            },
                        );
                    }
                    job.wrap.max_width = wrap_width;
                    ui.fonts_mut(|fonts| fonts.layout_job(job))
                };
                let mut output = TextEdit::multiline(&mut self.input)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .return_key(Some(shift_enter))
                    .interactive(!busy)
                    .layouter(&mut layouter)
                    .show(ui);
                let response = &output.response;

                // Wait until the widget is interactive again; requesting focus while
                // `busy` clears the flag on a disabled TextEdit and never restores it.
                if self.request_focus && !busy {
                    response.request_focus();
                    self.request_focus = false;
                }

                if !busy && response.has_focus() {
                    submitted = self.handle_keys(ui);
                }

                if self.cursor_to_end {
                    let end = CCursor::new(self.input.chars().count());
                    output
                        .state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(end)));
                    output.state.store(ui.ctx(), response.id);
                    self.cursor_to_end = false;
                }
            });

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                if self.transcript.is_empty() {
                    ui.label(RichText::new("Evaluate expressions here.").weak().italics());
                }
                for entry in &self.transcript {
                    match entry {
                        Entry::Input(text) => {
                            ui.label(RichText::new(format!("> {text}")).monospace());
                        }
                        Entry::Output(text) => {
                            ui.label(RichText::new(text.trim_end()).monospace());
                        }
                        Entry::Result(value) => {
                            ui.label(RichText::new(format!("=> {value}")).monospace().weak());
                        }
                        Entry::Error(text) => {
                            ui.colored_label(
                                egui::Color32::from_rgb(180, 40, 40),
                                RichText::new(text.trim_end()).monospace(),
                            );
                        }
                    }
                }
            });

        submitted
    }

    fn handle_keys(&mut self, ui: &egui::Ui) -> Option<String> {
        let enter = ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift);
        let up = ui.input(|i| i.key_pressed(Key::ArrowUp));
        let down = ui.input(|i| i.key_pressed(Key::ArrowDown));

        if enter {
            let trimmed = self.input.trim();
            if trimmed.is_empty() {
                self.input.clear();
            } else if is_complete(&self.input) {
                let source = self.input.trim_end().to_string();
                self.history.push(source.clone());
                self.history_pos = None;
                self.input.clear();
                self.request_focus = true;
                return Some(source);
            } else if !self.input.ends_with('\n') {
                self.input.push('\n');
            }
        } else if !self.input.contains('\n') {
            // Single-line only: leave Up/Down free for multi-line cursor motion.
            if up {
                self.history_prev();
            } else if down {
                self.history_next();
            }
        }
        None
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let next = match self.history_pos {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_pos = Some(next);
        self.input = self.history[next].clone();
        self.cursor_to_end = true;
    }

    fn history_next(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos + 1 >= self.history.len() {
            self.history_pos = None;
            self.input.clear();
        } else {
            self.history_pos = Some(pos + 1);
            self.input = self.history[pos + 1].clone();
        }
        self.cursor_to_end = true;
    }
}

/// Whether `input` is a complete top-level form (or a non-incomplete error).
pub fn is_complete(input: &str) -> bool {
    match tokenize(input, None) {
        Err(LexError::IncompleteString(_)) => false,
        Err(_) => true,
        Ok(tokens) => {
            let mut parser = Parser::with_tokens(tokens);
            loop {
                match parser.parse() {
                    Ok(None) => return true,
                    Ok(Some(_)) => continue,
                    Err(ParseError::IncompleteExpr(_)) => return false,
                    Err(_) => return true,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_complete_balanced() {
        assert!(is_complete("(+ 1 2)"));
        assert!(is_complete("(define (f x)\n  (* x 2))\n(f 3)"));
        assert!(is_complete(""));
        assert!(is_complete("   "));
    }

    #[test]
    fn is_complete_unbalanced_paren() {
        assert!(!is_complete("(+ 1"));
        assert!(!is_complete("(begin\n  (fd 10"));
    }

    #[test]
    fn is_complete_open_string() {
        assert!(!is_complete("(writeln \"hi"));
    }

    #[test]
    fn is_complete_other_parse_error() {
        // Unexpected close paren is complete so the evaluator can report the error.
        assert!(is_complete(")"));
    }
}
