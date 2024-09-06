use core::panic;

use crate::env::Env;
use crate::eval::eval;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;

pub fn load_prelude(env: &Env) {
    let primitives = [
        // nil
        "(define nil '())",
        // #t
        "(define #t 1)",
        // #f
        "(define #f '())",
        // else
        "(define else 1)",
        // null?
        "(define (null? lst) (eq? lst '()))",
        // if
        "(defmacro if (pred then else)`(cond (,pred ,then) (else ,else)))",
    ];

    let mut parser = Parser::new();

    for primitive in primitives {
        let mut tokens = Vec::new();
        let mut scanner = Scanner::new(primitive.chars());
        while let Some(token) = scanner
            .get_token()
            .expect(format!("Failed to tokenize prelude: {primitive}").as_str())
        {
            tokens.push(token);
        }

        parser.add_tokens(tokens);

        loop {
            match parser.parse() {
                Ok(expr) => {
                    let _ = eval(&expr, &env)
                        .expect(format!("Failed to evaluate prelude: {primitive}").as_str());
                }
                Err(ParseError::NeedMoreToken) => {
                    if parser.is_parsing() {
                        panic!("Failed to parse prelude - incomplete expression: {primitive}");
                    } else {
                        break; // we're done!
                    }
                }
                Err(ParseError::UnexpectedToken(token)) => {
                    panic!("Failed to parse prelude - unexpected token {token} in {primitive}");
                }
            }
        }
    }
}
