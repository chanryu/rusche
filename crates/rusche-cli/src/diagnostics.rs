use colored::Colorize;
use rusche::{Error, EvalError, Frame, LexError, ParseError, Span};
use unicode_width::UnicodeWidthStr;

use crate::host;

const MAX_HEAD_LINES: usize = 3;
const MAX_TAIL_LINES: usize = 1;

/// Everything needed to print a rustc-style diagnostic against a source buffer.
pub struct Diagnostic<'a> {
    pub source_name: &'a str,
    pub src: &'a str,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<&'a str>,
    pub trace: &'a [Frame],
}

/// Print an error message with optional source context, help, and call trace to stderr.
pub fn print_error(diag: &Diagnostic<'_>) {
    host::ensure_newline();

    eprintln!("{}: {}", "error".red(), diag.message);

    let Some(span) = diag.span else {
        print_help_and_trace(diag);
        return;
    };

    let lines: Vec<&str> = diag.src.lines().collect();
    if span.end.line >= lines.len() {
        print_help_and_trace(diag);
        return;
    }

    let loc = format!(
        "{}:{}:{}",
        diag.source_name,
        span.begin.line + 1,
        span.begin.column + 1
    );
    eprintln!("  {} {}", "-->".blue().bold(), loc);

    let print_line = |line: usize| {
        eprintln!(
            "{}{}",
            format!("{:>3}| ", line + 1).dimmed(),
            lines[line]
        );
    };

    // Context lines before the span.
    if span.begin.line >= 2 {
        print_line(span.begin.line - 2);
    }
    if span.begin.line >= 1 {
        print_line(span.begin.line - 1);
    }

    let span_line_count = span.end.line - span.begin.line + 1;
    let show_ellipsis = span_line_count > MAX_HEAD_LINES + MAX_TAIL_LINES;

    for (line, text) in lines
        .iter()
        .enumerate()
        .take(span.end.line + 1)
        .skip(span.begin.line)
    {
        let relative = line - span.begin.line;
        if show_ellipsis && relative >= MAX_HEAD_LINES && line + MAX_TAIL_LINES <= span.end.line {
            if relative == MAX_HEAD_LINES {
                eprintln!("{}", "   | ...".dimmed());
            }
            continue;
        }

        print_line(line);

        let begin_col = if line == span.begin.line {
            span.begin.column
        } else {
            text.chars().take_while(|c| c.is_whitespace()).count()
        };
        let end_col = if line == span.end.line {
            span.end.column
        } else {
            text.chars().count()
        };

        let prefix: String = text.chars().take(begin_col).collect();
        let marked: String = text
            .chars()
            .skip(begin_col)
            .take(end_col.saturating_sub(begin_col))
            .collect();
        let padding_width = UnicodeWidthStr::width(prefix.as_str());
        let caret_width = UnicodeWidthStr::width(marked.as_str()).max(1);

        // Preserve tabs in the padding so terminals align carets with tabstops;
        // otherwise pad by unicode display width so wide characters line up.
        let pad = if prefix.contains('\t') {
            prefix
                .chars()
                .map(|ch| if ch == '\t' { '\t' } else { ' ' })
                .collect::<String>()
        } else {
            " ".repeat(padding_width)
        };

        eprintln!(
            "{}{}{}",
            "   | ".dimmed(),
            pad,
            "^".repeat(caret_width).red()
        );
    }

    print_help_and_trace(diag);
}

/// Print a unified pipeline error ([`Error`]) against `src`.
pub fn print_pipeline_error(error: &Error, src: &str, source_name: &str) {
    print_error(&Diagnostic {
        source_name,
        src,
        message: error.message(),
        span: error.span(),
        help: error.help(),
        trace: error.trace(),
    });
}

/// Print an evaluation error against `src`.
pub fn print_eval_error(error: &EvalError, src: &str, source_name: &str) {
    print_error(&Diagnostic {
        source_name,
        src,
        message: error.message().to_string(),
        span: error.span,
        help: error.help(),
        trace: error.trace(),
    });
}

/// Print a lex error against `src`.
pub fn print_lex_error(error: &LexError, src: &str, source_name: &str) {
    print_error(&Diagnostic {
        source_name,
        src,
        message: error.message(),
        span: Some(error.span()),
        help: None,
        trace: &[],
    });
}

/// Print a parse error against `src`.
pub fn print_parse_error(error: &ParseError, src: &str, source_name: &str, span: Option<Span>) {
    print_error(&Diagnostic {
        source_name,
        src,
        message: error.message(),
        span: span.or(Some(error.span())),
        help: None,
        trace: &[],
    });
}

fn print_help_and_trace(diag: &Diagnostic<'_>) {
    if let Some(help) = diag.help {
        eprintln!("  {} {}", "=".blue().bold(), format!("help: {help}").dimmed());
    }

    for frame in diag.trace {
        let line = if let Some(span) = frame.call_site {
            format!(
                "in `{}`, called at {}:{}:{}",
                frame.name,
                diag.source_name,
                span.begin.line + 1,
                span.begin.column + 1
            )
        } else {
            format!("in `{}` (prelude)", frame.name)
        };
        eprintln!("  {} {}", "=".blue().bold(), line.dimmed());
    }
}
