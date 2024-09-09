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
    // map
    r#"
    (define (map fn lst)
        (if (null? lst)
            '()                          ; Base case: empty list
            (cons (fn (car lst))         ; Apply function to the first element
                  (map fn (cdr lst)))))  ; Recursive call on the rest of the list
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
    (define (append lst1 lst2)
        (if (null? lst1) lst2                             ; If lst1 is empty, return lst2
            (cons (car lst1) (append (cdr lst1) lst2))))  ; Otherwise, prepend the first element of lst1 and recurse
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
    (define (assoc key lst)
        (cond
            ((null? lst) #f)                       ; If the list is empty, return #f
            ((eq? (car (car lst)) key) (car lst))  ; If the car of the first element matches the key, return the pair
            (#t (assoc key (cdr lst)))))           ; Otherwise, recursively search the rest of the list
    "#,
    // subst
    r#"
    (define (subst new old lst)
        (cond
            ((null? lst) '())                                  ; If the list is empty, return an empty list
            ((eq? (car lst) old)                               ; If the first element matches 'old'
            (cons new (subst new old (cdr lst))))              ; Replace it with 'new' and recurse on the rest
            (#t (cons (car lst) (subst new old (cdr lst))))))  ; Otherwise, keep the first element and recurse
    "#,
    // reverse
    r#"
    (define (reverse lst)
        (if (null? lst) lst
            (append (reverse (cdr lst)) (list (car lst)))))
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
