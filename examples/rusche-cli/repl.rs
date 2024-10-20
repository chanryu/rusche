use rusche::{
    eval::Evaluator,
    lexer::{tokenize, LexError},
    parser::{ParseError, Parser},
};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::builtin::{load_io_procs, load_vec_procs};

pub fn run_repl() {
    print_logo();

    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");
    let mut parser = Parser::new();

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(evaluator.context());

    loop {
        let prompt = if parser.is_parsing() {
            "...... ❯ "
        } else {
            "rusche ❯ "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                match tokenize(&line) {
                    Ok(tokens) => parser.add_tokens(tokens),
                    Err(LexError::IncompleteString(span)) => {
                        println!("Error:{span}: Incomplete string");
                        continue;
                    }
                    Err(LexError::InvalidNumber(span)) => {
                        println!("Error:{span}: Invalid number");
                        continue;
                    }
                }

                loop {
                    match parser.parse() {
                        Ok(None) => {
                            break;
                        }
                        Ok(Some(expr)) => match evaluator.eval(&expr) {
                            Ok(result) => {
                                println!("; {}", result);
                            }
                            Err(error) => {
                                println!("Error: {}", error);
                            }
                        },
                        Err(ParseError::IncompleteExpr(_)) => break,
                        Err(ParseError::UnexpectedToken(token)) => {
                            parser.reset();
                            println!("Error: Unexpected token - {token}");
                        }
                    }
                }
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(error) => {
                println!("Error: {error}");
                break;
            }
        }
    }
}

fn print_logo() {
    println!(r"              ____                  __         ");
    println!(r"             / __ \__  ____________/ /_  ___   ");
    println!(r"            / /_/ / / / / ___/ ___/ __ \/ _ \  ");
    println!(r"Welcome to / _, _/ /_/ (__  ) /__/ / / /  __/ !");
    println!(r"          /_/ |_|\__,_/____/\___/_/ /_/\___/   ");
    println!(r"                                               ");
    println!(r"To exit, press Ctrl + D.                       ");
}
