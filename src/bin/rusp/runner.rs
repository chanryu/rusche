use crate::tokenize::tokenize;
use rusp::{
    env::Env,
    eval::eval,
    parser::{ParseError, Parser},
};

pub fn run_file(path: &str) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if let Err(e) = run_file_content(&content) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to read file at \"{path}\": {e}");
            std::process::exit(1);
        }
    }
}

fn run_file_content(text: &str) -> Result<(), String> {
    let mut parser =
        Parser::with_tokens(tokenize(text).map_err(|e| format!("Tokenization error: {}", e))?);
    let env = Env::new_root_env();

    loop {
        match parser.parse() {
            Ok(expr) => {
                let _ = eval(&expr, &env).map_err(|e| format!("Evaluation error: {}", e))?;
            }
            Err(ParseError::NeedMoreToken) => break,
            Err(e) => return Err(format!("Parsing error: {}", e)),
        }
    }

    if parser.is_parsing() {
        Err("Unexpected end of file.".to_owned())
    } else {
        Ok(())
    }
}
