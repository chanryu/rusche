mod builtin;
mod repl;

use colored::Colorize;
use rusche::{tokenize, Evaluator, LexError, ParseError, Parser, Span};

use builtin::{load_io_procs, load_vec_procs};
use repl::run_repl;

fn main() {
    let mut args = std::env::args().skip(1); // skip the program name

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(evaluator.context());

    if let Some(path) = args.next() {
        run_file(evaluator, &path);
    } else {
        run_repl(evaluator);
    }
}

fn run_file(evaluator: Evaluator, path: &str) {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let tokens = match tokenize(&text, None) {
                Ok(tokens) => tokens,
                Err(error) => match error {
                    LexError::InvalidNumber(span) => {
                        print_error("invalid number", &text_to_lines(&text), Some(span));
                        return;
                    }
                    LexError::IncompleteString(span) => {
                        print_error("incomplete string", &text_to_lines(&text), Some(span));
                        return;
                    }
                },
            };

            let mut parser = Parser::with_tokens(tokens);
            loop {
                match parser.parse() {
                    Ok(None) => {
                        break;
                    }
                    Ok(Some(expr)) => match evaluator.eval(&expr) {
                        Ok(_) => {}
                        Err(e) => {
                            print_error(&e.message, &text_to_lines(&text), e.span);
                            break;
                        }
                    },
                    Err(ParseError::IncompleteExpr(_)) => {
                        eprintln!("Failed to parse - incomplete expression");
                        break;
                    }
                    Err(ParseError::UnexpectedToken(token)) => {
                        print_error(
                            &format!("unexpected token - \"{token}\""),
                            &text_to_lines(&text),
                            Some(token.span()),
                        );
                        break;
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read file at \"{path}\": {e}"),
    }
}

fn text_to_lines(text: &str) -> Vec<String> {
    text.lines().map(|line| line.to_string()).collect()
}

fn print_error(message: &str, lines: &Vec<String>, span: Option<Span>) {
    println!("{}: {}", "error".red(), message);

    let Some(span) = span else { return };

    if span.begin.line < lines.len() {
        if span.begin.line > 0 {
            println!("  {:03}: {}", span.begin.line, lines[span.begin.line - 1]);
        }
        println!("  {:03}: {}", span.begin.line + 1, lines[span.begin.line]);
        println!(
            "       {}{}",
            " ".repeat(span.begin.column),
            "^".repeat(span.end.column - span.begin.column).red()
        );
    }
}
