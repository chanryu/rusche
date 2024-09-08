use core::panic;

use crate::env::Env;
use crate::eval::eval;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;

const PIECES: [&str; 9] = [
    // #t, #f
    r#"
    (define #t 1)
    (define #f '())
    "#,
    // caar, cadr, cadar, caddr, cdar
    r#"
    (define (caar  lst) (car (car lst)))
    (define (cadr  lst) (cdr (car lst)))
    (define (cadar lst) (car (cdr (car lst))))
    (define (caddr lst) (car (cdr (cdr lst))))
    (define (cdar  lst) (car (cdr lst)))
    "#,
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
    // map
    r#"
    (define (map fn lst)
        (if (null? lst)
            '()                          ; Base case: empty list
            (cons (fn (car lst))         ; Apply function to the first element
                  (map fn (cdr lst)))))  ; Recursive call on the rest of the list
    "#,
    // let
    r#"
    (defmacro let (bindings *body)
        `((lambda ,(map car bindings) ; Get the list of variable names
             ,@body)                  ; The body of the let becomes the lambda's body
          ,@(map cdar bindings)))     ; Apply the values to the lambda
    "#,
    // begin
    r#"
    (defmacro begin (*exprs)
        `(let () ,@exprs))
    "#,
    // and, or, not
    r#"
    (define (and x y) (if x (if y #t #f) #f))
    (define (or  x y) (if x #t (if y #t #f)))
    (define (not x  ) (if x #f #t))
    "#,
    // null?
    r#"
    (define (null? e) (eq? e '()))
    "#,
];

pub fn load_prelude(env: &Env) {
    let mut parser = Parser::new();

    for piece in PIECES {
        let mut scanner = Scanner::new(piece.chars());
        let tokens = std::iter::from_fn(|| match scanner.get_token() {
            Ok(Some(token)) => Some(token),
            Ok(None) => None,
            Err(_) => panic!("Failed to tokenize prelude: {piece}"),
        })
        .collect::<Vec<_>>();

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
