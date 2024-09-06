use core::panic;

use crate::env::Env;
use crate::eval::eval;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;

const PRELUDE: &str = r#"

;; nil
(define nil '())

;; #t, #f
(define #t '#t)
(define #f '())

;; else
(define else #t)

;; null?
(define (null? lst)
  (eq? lst '()))

;; if
(defmacro if (pred then else)
  `(cond (,pred ,then) (else ,else)))

"#;

pub fn load_prelude(env: &Env) {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(PRELUDE.chars());
    while let Some(token) = scanner.get_token().expect("Prelude failure!") {
        tokens.push(token);
    }

    let mut parser = Parser::new();
    parser.add_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(expr) => {
                let _ = eval(&expr, &env).expect("Prelude eval failure!");
            }
            Err(ParseError::NeedMoreToken) => {
                if parser.is_parsing() {
                    panic!("Prelude parse failure!");
                } else {
                    break; // we're done!
                }
            }
            Err(ParseError::UnexpectedToken(_)) => {
                panic!("Prelude parse failure!");
            }
        }
    }
}
