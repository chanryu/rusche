mod builtin;
mod repl;

use rusche::{
    eval::Evaluator,
    lexer::{tokenize, LexError},
    parser::{ParseError, Parser},
    span::Loc,
};

use builtin::{load_io_procs, load_vec_procs};
use repl::run_repl;

fn main() {
    let mut args = std::env::args().skip(1); // skip the program name

    if let Some(path) = args.next() {
        run_file(&path);
    } else {
        run_repl();
    }
}

fn run_file(path: &str) {
    match std::fs::read_to_string(path) {
        Ok(text) => run_file_content(&text),
        Err(e) => eprintln!("Failed to read file at \"{path}\": {e}"),
    }
}

fn run_file_content(text: &str) {
    let tokens = match tokenize(text, Loc::default()) {
        Ok(tokens) => tokens,
        Err(LexError::IncompleteString(span)) => {
            eprintln!("Error:{span}: Incomplete string");
            return;
        }
        Err(LexError::InvalidNumber(span)) => {
            eprintln!("Error:{span}: Invalid number");
            return;
        }
    };

    let evaluator = Evaluator::with_prelude();

    load_io_procs(evaluator.context());
    load_vec_procs(evaluator.context());

    let mut parser = Parser::with_tokens(tokens);
    loop {
        match parser.parse() {
            Ok(None) => {
                break;
            }
            Ok(Some(expr)) => match evaluator.eval(&expr) {
                Ok(_) => {}
                Err(e) => match e.span {
                    Some(span) => eprintln!("Error: {}:{}", span, e.message),
                    None => eprintln!("Error: {}", e.message),
                },
            },
            Err(ParseError::IncompleteExpr(_)) => {
                eprintln!("Failed to parse - incomplete expression");
                break;
            }
            Err(ParseError::UnexpectedToken(token)) => {
                eprintln!("Error: Unexpected token - {token}");
                break;
            }
        }
    }
}
