mod builtin;
mod repl;

use colored::Colorize;
use rusche::{tokenize, Evaluator, LexError, Loc, ParseError, Parser, Span};

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
                        print_error("invalid number", &text, Some(span));
                        return;
                    }
                    LexError::IncompleteString(span) => {
                        print_error("incomplete string", &text, Some(span));
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
                            print_error(&e.message, &text, e.span);
                            break;
                        }
                    },
                    Err(ParseError::IncompleteExpr(token)) => {
                        let begin_loc = token.span().begin;
                        let end_loc =
                            Loc::new(text.lines().count() - 1, text.lines().last().unwrap().len());
                        print_error(
                            "incomplete expression",
                            &text,
                            Some(Span::new(begin_loc, end_loc)),
                        );
                        break;
                    }
                    Err(ParseError::UnexpectedToken(token)) => {
                        print_error(
                            &format!("unexpected token: \"{token}\""),
                            &text,
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

fn print_error(message: &str, src: &str, span: Option<Span>) {
    let lines: Vec<&str> = src.lines().collect();

    println!("{}: {}", "error".red(), message);

    let Some(span) = span else { return };

    if span.end.line < lines.len() {
        let print_line =
            |line| println!("{}{}", format!("{:>3}| ", line + 1).dimmed(), lines[line]);
        if span.begin.line >= 2 {
            print_line(span.begin.line - 2);
        }
        if span.begin.line >= 1 {
            print_line(span.begin.line - 1);
        }

        for line in span.begin.line..span.end.line + 1 {
            print_line(line);

            let begin_col = if line == span.begin.line {
                span.begin.column
            } else {
                lines[line]
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count()
            };
            let end_col = if line == span.end.line {
                span.end.column
            } else {
                lines[line].len()
            };
            println!(
                "{}{}{}",
                "   | ".dimmed(),
                " ".repeat(begin_col),
                "^".repeat(end_col - begin_col).red()
            );
        }
    }
}
