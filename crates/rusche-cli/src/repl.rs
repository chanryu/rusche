use colored::Colorize;
use rusche::{tokenize, Evaluator, ParseError, Parser};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::diagnostics::print_error;

pub fn run_repl(evaluator: Evaluator) {
    print_logo();

    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");

    let mut src = String::new();
    let mut consumed_lines: usize = 0;

    let mut parser = Parser::new();
    loop {
        let line = src.lines().count();
        let prompt = if line == consumed_lines {
            format!("repl:{:03}❯ ", line + 1)
        } else {
            debug_assert!(parser.is_parsing());
            format!("....:{:03}❯ ", line + 1)
        };

        match rl.readline(&prompt) {
            Ok(text) => {
                let _ = rl.add_history_entry(text.as_str());
                let loc = Some(rusche::Loc::new(line, 0));
                let res = tokenize(&text, loc);

                match res {
                    Ok(tokens) => parser.add_tokens(tokens),
                    Err(err) => {
                        let error_src = src.clone() + &text;
                        print_error(&err, &error_src, Some(err.span()));
                        continue;
                    }
                }

                src.push_str(&text);
                src.push('\n');

                loop {
                    match parser.parse() {
                        Ok(None) => {
                            consumed_lines = src.lines().count();
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
                        Err(error) => {
                            parser.reset();
                            print_error(&error, &src, Some(error.span()));
                            consumed_lines = src.lines().count();
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
