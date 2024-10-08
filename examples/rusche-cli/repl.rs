use rusche::{
    eval::Evaluator,
    lexer::tokenize,
    parser::{ParseError, Parser},
};
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::builtin::{load_io_procs, load_vec_procs};

pub fn run_repl() {
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
                    Err(error) => {
                        println!("Error: {}", error);
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
                                println!(
                                    "unreachable envs: {}",
                                    evaluator.count_unreachable_envs()
                                );
                            }
                            Err(error) => {
                                println!("Error: {}", error);
                            }
                        },
                        Err(ParseError::NeedMoreToken) => break,
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
