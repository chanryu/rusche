mod scanner;

use scanner::{ScanError, Scanner};

fn main() {
    let text = r#"('() ) .123  ( abc!)'(("abc?"))"xd"#;
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
