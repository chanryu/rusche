use colored::Colorize;
use rusche::Span;
use std::fmt::Display;

const MAX_HEAD_LINES: usize = 3;
const MAX_TAIL_LINES: usize = 1;

/// Print an error message with optional source context to stderr.
pub fn print_error(message: &dyn Display, src: &str, span: Option<Span>) {
    eprintln!("{}: {}", "error".red(), message);

    let Some(span) = span else { return };

    let lines: Vec<&str> = src.lines().collect();
    if span.end.line >= lines.len() {
        return;
    }

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
        if show_ellipsis
            && relative >= MAX_HEAD_LINES
            && line + MAX_TAIL_LINES <= span.end.line
        {
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

        let caret_len = end_col.saturating_sub(begin_col).max(1);
        let mut padding = String::new();
        for (i, ch) in text.chars().enumerate() {
            if i >= begin_col {
                break;
            }
            padding.push(if ch == '\t' { '\t' } else { ' ' });
        }

        eprintln!(
            "{}{}{}",
            "   | ".dimmed(),
            padding,
            "^".repeat(caret_len).red()
        );
    }
}
