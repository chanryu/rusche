use rusche::{
    eval::Evaluator,
    lexer::tokenize,
    parser::{ParseError, Parser},
};

use crate::builtin::{load_io_procs, load_vec_procs};

pub fn run_file(path: &str) {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            if let Err(e) = run_file_content(&text) {
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
    let tokens = tokenize(text).map_err(|e| format!("Tokenization error: {}", e))?;
    let mut parser = Parser::with_tokens(tokens);

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(evaluator.context());

    loop {
        match parser.parse() {
            Ok(None) => {
                break;
            }
            Ok(Some(expr)) => {
                let _ = evaluator
                    .eval(&expr)
                    .map_err(|e| format!("Evaluation error: {}", e))?;
            }
            Err(ParseError::NeedMoreToken) => {
                return Err("Failed to parse - incomplete expression".to_string());
            }
            Err(e) => return Err(format!("Parsing error: {}", e)),
        }
    }

    Ok(())
}
