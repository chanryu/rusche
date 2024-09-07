use core::panic;

use crate::env::Env;
use crate::eval::eval;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;

const PIECES: [&str; 9] = [
    // null
    "(define null '())",
    // #t
    "(define #t 1)",
    // #f
    "(define #f '())",
    // if
    r#"
    (defmacro if (pred then else)
        `(cond (,pred ,then)
               (#t    ,else)))
    "#,
    // list
    r#"
    (defmacro list (*args)
        (if (null? args)
            '()
            `(cons ,(car args) (list ,@(cdr args)))))
    "#,
    // null?
    "(define (null? e) (eq? e null))",
    // and
    "(define (and x y) (if x (if y #t #f) #f))",
    // or
    "(define (or x y) (if x #t (if y #t #f)))",
    // not
    "(define (not x) (if x #f #t))",
];

pub fn load_prelude(env: &Env) {
    let mut parser = Parser::new();

    for piece in PIECES {
        let mut tokens = Vec::new();
        let mut scanner = Scanner::new(piece.chars());
        while let Some(token) = scanner
            .get_token()
            .expect(format!("Failed to tokenize prelude: {piece}").as_str())
        {
            tokens.push(token);
        }

        parser.add_tokens(tokens);

        loop {
            match parser.parse() {
                Ok(expr) => {
                    let _ = eval(&expr, &env)
                        .expect(format!("Failed to evaluate prelude: {piece}").as_str());
                }
                Err(ParseError::NeedMoreToken) => {
                    if parser.is_parsing() {
                        panic!("Failed to parse prelude - incomplete expression: {piece}");
                    } else {
                        break; // we're done!
                    }
                }
                Err(ParseError::UnexpectedToken(token)) => {
                    panic!("Failed to parse prelude - unexpected token {token} in {piece}");
                }
            }
        }
    }
}
