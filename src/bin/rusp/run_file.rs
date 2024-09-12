use crate::tokenize::tokenize;
use rusp::{
    env::Env,
    eval::eval,
    parser::{ParseError, Parser},
};

pub fn run_file(file_path: &str) {
    match std::fs::read_to_string(file_path) {
        Ok(contents) => match tokenize(&contents) {
            Ok(tokens) => {
                let env = Env::new_root_env();

                let mut parser = Parser::new();
                parser.add_tokens(tokens);
                loop {
                    match parser.parse() {
                        Ok(expr) => match eval(&expr, &env) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        },
                        Err(ParseError::NeedMoreToken) => break,
                        Err(e) => {
                            eprintln!("Parse Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                if parser.is_parsing() {
                    eprintln!("Unexpected end of file.");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                println!("Tokenize Error: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    }
}
