use rusp::env::Env;
use rusp::eval::eval;
use rusp::parser::{ParseError, Parser};
use rusp::scanner::{Scanner, TokenError};
use rusp::token::Token;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub fn repl() {
    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");

    print_logo();

    let env = Env::new_root_env();
    let mut parser = Parser::new();

    loop {
        let prompt = if parser.is_parsing() {
            ".... > "
        } else {
            "rusp > "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                match tokenize(&line) {
                    Ok(tokens) => parser.add_tokens(tokens),
                    Err(error) => {
                        println!("Error: {}", error);
                        continue;
                    }
                }

                loop {
                    match parser.parse() {
                        Ok(expr) => match eval(&expr, &env) {
                            Ok(result) => {
                                println!("; {}", result);
                            }
                            Err(error) => {
                                println!("; Error: {}", error);
                            }
                        },
                        Err(ParseError::NeedMoreToken) => break,
                        Err(error) => {
                            parser.reset();
                            println!("; Error: {}", error);
                        }
                    }
                }
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(error) => {
                println!("Error: {:?}", error);
                break;
            }
        }
    }
}

fn print_logo() {
    println!("           ┬─┐┬ ┬┌─┐┌─┐");
    println!("Welcome to ├┬┘│ │└─┐├─┘");
    println!("           ┴└─└─┘└─┘┴  !");
}

fn tokenize(text: &str) -> Result<Vec<Token>, TokenError> {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());

    loop {
        match scanner.get_token()? {
            Some(token) => tokens.push(token),
            None => return Ok(tokens),
        }
    }
}
