mod builtins;
mod env;
mod eval;
mod expr;
mod parser;
mod scanner;

use env::Env;
use eval::eval;
use parser::Parser;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    let env = Env::new_root_env();
    loop {
        match rl.readline("rusp >> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                let mut parser = Parser::new(line.chars());

                match parser.parse() {
                    Ok(expr) => match eval(&expr, &env) {
                        Ok(result) => {
                            println!("{} => {}", expr, result);
                        }
                        Err(error) => {
                            println!("Error: {}", error);
                        }
                    },
                    Err(error) => println!("Error: {}", error),
                }
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
