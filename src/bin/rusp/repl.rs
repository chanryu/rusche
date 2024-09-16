use crate::tokenize::tokenize;
use rusp::{
    eval::EvalContext,
    parser::{ParseError, Parser},
};
use rustyline::{error::ReadlineError, DefaultEditor};

pub fn run_repl() {
    print_logo();

    let mut rl = DefaultEditor::new().expect("Failed to initialize line reader!");
    let context = EvalContext::new();
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
                        Ok(expr) => match context.eval(&expr) {
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
