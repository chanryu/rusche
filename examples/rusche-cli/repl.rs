use colored::Colorize;
use rusche::{tokenize, Evaluator, LexError, Loc, ParseError, Parser};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::print_error;

pub fn run_repl(evaluator: Evaluator) {
    print_logo();

    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");

    let mut src = String::new();

    let mut parser = Parser::new();
    loop {
        let line = src.lines().count();
        let prompt = if line == 0 {
            format!("rusche:{:02}❯ ", line + 1)
        } else {
            debug_assert!(parser.is_parsing());
            format!("......:{:02}❯ ", line + 1)
        };

        match rl.readline(&prompt) {
            Ok(text) => {
                let _ = rl.add_history_entry(text.as_str());
                let loc = Some(Loc::new(line, 0));
                let res = tokenize(&text, loc);

                match res {
                    Ok(tokens) => parser.add_tokens(tokens),
                    Err(err) => {
                        let error_src = src.clone() + &text;
                        match err {
                            LexError::InvalidNumber(span) => {
                                print_error("invalid number", &error_src, Some(span))
                            }
                            LexError::IncompleteString(span) => {
                                print_error("incomplete string", &error_src, Some(span))
                            }
                        }
                        continue;
                    }
                }

                src.push_str(&text);
                src.push_str("\n");

                loop {
                    match parser.parse() {
                        Ok(None) => {
                            src.clear();
                            break;
                        }
                        Ok(Some(expr)) => match evaluator.eval(&expr) {
                            Ok(result) => {
                                println!("{}", result.to_string().green());
                            }
                            Err(error) => {
                                print_error(&error.message, &src, error.span);
                            }
                        },
                        Err(ParseError::IncompleteExpr(_)) => break,
                        Err(ParseError::UnexpectedToken(token)) => {
                            parser.reset();
                            print_error(
                                &format!("unexpected token: \"{token}\""),
                                &src,
                                Some(token.span()),
                            );
                            src.clear();
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
