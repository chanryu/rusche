mod builtin;
mod diagnostics;
mod repl;

use std::process::ExitCode;

use rusche::{tokenize, Evaluator, Loc, ParseError, Parser, Span};

use builtin::{load_io_procs, load_scheme_aliases, load_vec_procs};
use diagnostics::print_error;
use repl::run_repl;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1); // skip the program name

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(&evaluator);
    load_scheme_aliases(&evaluator);

    if let Some(path) = args.next() {
        match std::fs::read_to_string(&path) {
            Ok(text) => match run_source(&evaluator, &text) {
                Ok(()) => ExitCode::SUCCESS,
                Err(()) => ExitCode::from(1),
            },
            Err(e) => {
                eprintln!("Failed to read file at \"{path}\": {e}");
                ExitCode::from(2)
            }
        }
    } else {
        run_repl(evaluator);
        ExitCode::SUCCESS
    }
}

/// Exit codes for script evaluation: `Ok` on success, `Err` on lex/parse/eval failure.
fn run_source(evaluator: &Evaluator, text: &str) -> Result<(), ()> {
    let (body, loc) = strip_shebang(text);
    let tokens = match tokenize(body, Some(loc)) {
        Ok(tokens) => tokens,
        Err(error) => {
            print_error(&error, text, Some(error.span()));
            return Err(());
        }
    };

    let mut parser = Parser::with_tokens(tokens);
    loop {
        match parser.parse() {
            Ok(None) => return Ok(()),
            Ok(Some(expr)) => match evaluator.eval(&expr) {
                Ok(_) => {}
                Err(e) => {
                    print_error(&e.message, text, e.span);
                    return Err(());
                }
            },
            Err(error @ ParseError::IncompleteExpr(_)) => {
                let begin_loc = error.span().begin;
                let end_line = text.lines().count().saturating_sub(1);
                let end_col = text.lines().last().map(|l| l.chars().count()).unwrap_or(0);
                let span = if begin_loc.line < text.lines().count() {
                    Some(Span::new(
                        begin_loc,
                        Loc::new(end_line, end_col.max(begin_loc.column + 1)),
                    ))
                } else {
                    Some(error.span())
                };
                print_error(&error, text, span);
                return Err(());
            }
            Err(error) => {
                print_error(&error, text, Some(error.span()));
                return Err(());
            }
        }
    }
}

fn strip_shebang(text: &str) -> (&str, Loc) {
    if text.starts_with("#!") {
        if let Some(pos) = text.find('\n') {
            (&text[pos + 1..], Loc::new(1, 0))
        } else {
            ("", Loc::new(1, 0))
        }
    } else {
        (text, Loc::default())
    }
}
