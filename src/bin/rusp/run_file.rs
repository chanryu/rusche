use crate::tokenize::tokenize;
use rusp::{
    env::Env,
    eval::eval,
    parser::{ParseError, Parser},
};

pub fn run_file(file_path: &str) {
    match std::fs::read_to_string(file_path) {
        Ok(contents) => {
            if let Err(e) = run_file_contents(&contents) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn run_file_contents(text: &str) -> Result<(), String> {
    match tokenize(&text) {
        Ok(tokens) => {
            let env = Env::new_root_env();

            let mut parser = Parser::new();
            parser.add_tokens(tokens);
            loop {
                match parser.parse() {
                    Ok(expr) => match eval(&expr, &env) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(format!("{}", e));
                        }
                    },
                    Err(ParseError::NeedMoreToken) => break,
                    Err(e) => {
                        return Err(format!("{}", e));
                    }
                }
            }
            if parser.is_parsing() {
                Err("Unexpected end of file.".to_owned())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("{}", e)),
    }
}
