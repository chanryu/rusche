use crate::env::Env;
use crate::eval::eval;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;

const PIECES: [&str; 15] = [
    // #t, #f
    r#"
    (define #t 1)
    (define #f '())
    "#,
    // caar, cadr, cdar, cdar
    r#"
    (define (caar  lst) (car (car lst)))
    (define (cadr  lst) (cdr (car lst)))
    (define (cdar  lst) (car (cdr lst)))
    (define (cddr  lst) (cdr (cdr lst)))
    "#,
    // cadar
    r#"
    (define (cadar lst) (car (cadr lst)))
    (define (caddr lst) (cdr (cadr lst)))
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
    // append
    r#"
    (define (append lst lst2)
        (if (null? lst) lst2
	        (cons (car lst) (append (cdr lst) lst2))))
    "#,
    // pair
    r#"
    (define (pair lst1 lst2)
        (cond ((and (null? lst1) (null? lst2)) '())
              ((and (not (atom? lst1)) (not (atom? lst2)))
               (cons (cons (car lst1) (cons (car lst2) '()))
                     (pair (cdr lst1) (cdr lst2))))))
    "#,
    // assoc
    r#"
    (define (assoc x y)
        (if (eq? (caar y) x) (cadar y)
		    (assoc x (cdr y))))
    "#,
    // subst
    r#"
    (define (subst x y z)
        (if (atom? z) (if (eq z y) x z)
		    (cons (subst x y (car z)) (subst x y (cdr z)))))
    "#,
    // reverse
    r#"
    (define (reverse lst)
        (if (null? lst) lst
            (append (reverse (cdr lst)) (car lst))))
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
