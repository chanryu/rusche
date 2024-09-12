use rusp::{
    env::Env,
    eval::eval,
    parser::{ParseError, Parser},
    scanner::{Scanner, TokenError},
    token::Token,
};
use rustyline::{error::ReadlineError, DefaultEditor};

fn main() {
    if let Some(file_path) = std::env::args().skip(1).next() {
        run_file(&file_path);
    } else {
        // no filename is give. fallback to REPL
        run_repl();
    }
}

fn run_file(file_path: &str) {
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

fn run_repl() {
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
