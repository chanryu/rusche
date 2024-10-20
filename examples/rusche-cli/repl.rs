use colored::Colorize;
use rusche::{tokenize, Evaluator, LexError, Loc, ParseError, Parser, Span};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::builtin::{load_io_procs, load_vec_procs};

pub fn run_repl() {
    print_logo();

    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(evaluator.context());

    let mut lines = Vec::new();

    let mut parser = Parser::new();
    loop {
        let prompt = if parser.is_parsing() {
            &format!("{:06}❯ ", lines.len() + 1)
        } else {
            "rusche❯ "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let loc = Some(Loc::new(lines.len(), 0));
                let res = tokenize(&line, loc);

                lines.push(line);

                match res {
                    Ok(tokens) => parser.add_tokens(tokens),
                    Err(err) => {
                        match err {
                            LexError::InvalidNumber(span) => {
                                print_error("invalid number", &lines, Some(span))
                            }
                            LexError::IncompleteString(span) => {
                                print_error("incomplete string", &lines, Some(span))
                            }
                        }
                        lines.pop();
                        continue;
                    }
                }

                loop {
                    match parser.parse() {
                        Ok(None) => {
                            lines.clear();
                            break;
                        }
                        Ok(Some(expr)) => match evaluator.eval(&expr) {
                            Ok(result) => {
                                println!("{}", result.to_string().green());
                            }
                            Err(error) => {
                                print_error(&error.message, &lines, error.span);
                            }
                        },
                        Err(ParseError::IncompleteExpr(_)) => break,
                        Err(ParseError::UnexpectedToken(token)) => {
                            parser.reset();
                            print_error(
                                &format!("unexpected token - {token}"),
                                &lines,
                                Some(token.span()),
                            );
                            lines.clear();
                        }
                    }
                }
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(error) => {
                eprintln!("{error}");
                break;
            }
        }
    }
}

#[rustfmt::skip]
fn print_logo() {
    println!("          {}  ", r"    ____                  __       ".bold().cyan());
    println!("          {}  ", r"   / __ \__  ____________/ /_  ___ ".bold().cyan());
    println!("          {}  ", r"  / /_/ / / / / ___/ ___/ __ \/ _ \".bold().cyan());
    println!("Welcome to{} !", r" / _, _/ /_/ (__  ) /__/ / / /  __/".bold().cyan());
    println!("          {}  ", r"/_/ |_|\__,_/____/\___/_/ /_/\___/ ".bold().cyan());

    println!("\n{}", "To exit, press Ctrl + D.".dimmed());
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
