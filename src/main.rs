mod scanner;

use scanner::Scanner;

use crate::scanner::ScanError;

fn main() {
    let text = r#"('() )   
    ( abc!)'(("abc?"))"#;
    let mut scanner = Scanner::new(text.chars());

    loop {
        match scanner.get_token() {
            Ok(token) => println!("token: {:?}", token),
            Err(ScanError::EndOfFile) => {
                println!("EOF");
                break;
            }
            Err(err) => {
                println!("error: {:?}", err);
            }
        }
    }
}
