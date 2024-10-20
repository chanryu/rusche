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
                        // let lines: Vec<String> =
                        //     text.lines().map(|line| line.to_string()).collect();
                        // let begin_loc = token.span().begin;
                        // let end_loc = Loc::new(lines.len() - 1, lines.last().unwrap().len());
                        // print_error_lines(
                        //     "incomplete expression",
                        //     &lines,
                        //     Some(Span::new(begin_loc, end_loc)),
                        // );
                        print_error("incomplete expression", &text, Some(token.span()));
                        break;
                    }
                    Err(ParseError::UnexpectedToken(token)) => {
                        print_error(
                            &format!("unexpected token - \"{token}\""),
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
    let lines = src.lines().map(|line| line.to_string()).collect();
    print_error_lines(message, &lines, span);
}

fn print_error_lines(message: &str, lines: &Vec<String>, span: Option<Span>) {
    println!("{}: {}", "error".red(), message);

    let Some(span) = span else { return };

    if span.end.line < lines.len() {
        let print_line =
            |line| println!("{}{}", format!("{:>4}| ", line + 1).dimmed(), lines[line]);
        if span.begin.line >= 2 {
            print_line(span.begin.line - 2);
        }
        if span.begin.line >= 1 {
            print_line(span.begin.line - 1);
        }

        print_line(span.begin.line);
        println!(
            "{}{}{}",
            "    | ".dimmed(),
            " ".repeat(span.begin.column),
            "^".repeat(span.end.column - span.begin.column).red()
        );
    }
}
