use crate::{
    eval::{eval, EvalContext},
    lexer::tokenize,
    parser::{ParseError, Parser},
};

const PRELUDE_SYMBOLS: [&str; 4] = [
    // #t
    "(define #t 1)",
    // #f
    "(define #f '())",
    // numeric operation aliases
    r#"
    (define + num-add)
    (define - num-subtract)
    (define * num-multiply)
    (define / num-divide)
    (define % num-modulo)
    (define < num-less)
    (define > num-greater)
    "#,
    // = (eq? alias)
    "(define = eq?)",
];

const PRELUDE_MACROS: [&str; 8] = [
    // begin
    r#"
    (defmacro (begin . exprs)
        `((lambda () ,@exprs)))
    "#,
    // cond
    r#"
    (defmacro (cond . clauses)
        (if (null? clauses)
            #f                                          ; No more clauses, return #f by default
            (let ((clause (car clauses)))
                (if (eq? (car clause) 'else)            ; If the first clause is 'else'
                    `(begin ,@(cdr clause))             ; Expand to the else expression(s)
                    `(if ,(car clause)                  ; Otherwise, expand to an if expression
                        (begin ,@(cdr clause))          ; If condition is true, evaluate the body
                        (cond ,@(cdr clauses)))))))     ; Else, recursively process remaining clauses
    "#,
    // defun
    r#"
    (defmacro (defun name args . body)
        `(define ,name (lambda ,args ,@body)))
    "#,
    // let
    r#"
    (defmacro (let bindings . body)
        `((lambda ,(map car bindings) ; Get the list of variable names
             ,@body)                  ; The body of the let becomes the lambda's body
          ,@(map cadr bindings)))     ; Apply the values to the lambda
    "#,
    // list
    r#"
    (defmacro (list . args)
        (if (null? args)
            '()
            `(cons ,(car args) (list ,@(cdr args)))))
    "#,
    // while -- the helper `loop` is scoped inside a lambda so it does not leak into the caller
    r#"
    (defmacro (while condition . body)
        `((lambda ()
            (define (loop)
                (if ,condition (begin ,@body (loop))))
            (loop))))
    "#,
    // and -- short-circuits, returns the last operand or #f
    r#"
    (defmacro (and . args)
        (cond ((null? args) #t)
              ((null? (cdr args)) (car args))
              (else `(if ,(car args) (and ,@(cdr args)) #f))))
    "#,
    // or -- short-circuits, returns the first truthy operand or #f
    r#"
    (defmacro (or . args)
        (cond ((null? args) #f)
              ((null? (cdr args)) (car args))
              (else `((lambda (or-value)
                        (if or-value or-value (or ,@(cdr args))))
                      ,(car args)))))
    "#,
];

const PRELUDE_FUNCS: [&str; 10] = [
    // caar, cadr, cdar, cddr
    r#"
    (define (caar lst) (car (car lst)))
    (define (cadr lst) (car (cdr lst)))
    (define (cdar lst) (cdr (car lst)))
    (define (cddr lst) (cdr (cdr lst)))
    "#,
    // not
    r#"
    (define (not x) (if x #f #t))
    "#,
    // null?
    r#"
    (define (null? e) (eq? e '()))
    "#,
    // reverse -- tail-recursive so long lists do not hit the call depth limit
    r#"
    (define (reverse lst)
        (define (loop lst acc)
            (if (null? lst) acc (loop (cdr lst) (cons (car lst) acc))))
        (loop lst '()))
    "#,
    // map -- tail-recursive
    r#"
    (define (map fn lst)
        (define (loop lst acc)
            (if (null? lst)
                (reverse acc)
                (loop (cdr lst) (cons (fn (car lst)) acc))))
        (loop lst '()))
    "#,
    // append -- tail-recursive
    r#"
    (define (append lst1 lst2)
        (define (loop lst acc)
            (if (null? lst) acc (loop (cdr lst) (cons (car lst) acc))))
        (loop (reverse lst1) lst2))
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
    // numeric operations
    r#"
    (define (<= x y) (or (< x y) (= x y)))
    (define (>= x y) (or (> x y) (= x y)))
    "#,
];

pub fn load_prelude(context: &EvalContext) {
    for src in PRELUDE_SYMBOLS {
        eval_src(src, context);
    }
    for src in PRELUDE_MACROS {
        eval_src(src, context);
    }
    for src in PRELUDE_FUNCS {
        eval_src(src, context);
    }
}

fn eval_src(src: &str, context: &EvalContext) {
    let tokens =
        tokenize(src, None).unwrap_or_else(|_| panic!("Prelude tokniization failed: {}", src));

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(None) => {
                break; // we're done!
            }
            Ok(Some(expr)) => {
                // Prelude source locations are meaningless to users; strip them so errors
                // raised inside prelude code are attributed to the user's call site instead.
                let expr = expr.without_spans();
                let _ = eval(&expr, context)
                    .unwrap_or_else(|_| panic!("Prelude evaluation failed: {}", src));
            }
            Err(ParseError::IncompleteExpr(_)) => {
                panic!("Prelude parse failure - incomplete expression: {}", src);
            }
            Err(ParseError::UnexpectedToken(token)) => {
                panic!(
                    "Prelude parse failure - unexpected token \"{}\": {}",
                    token, src
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;

    #[test]
    fn test_eval_src() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1)", e.context()); // no panic
    }

    #[test]
    #[should_panic(expected = "Prelude tokniization failed: \"x")]
    fn test_eval_src_invalid_string() {
        let e = Evaluator::with_builtin();
        eval_src("\"x", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude evaluation failed: (x)")]
    fn test_eval_src_eval_failed() {
        let e = Evaluator::with_builtin();
        eval_src("(x)", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude parse failure - incomplete expression: (define x 1")]
    fn test_eval_src_incomplete_expr() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1", e.context());
    }

    #[test]
    #[should_panic(expected = "Prelude parse failure - unexpected token \")\": (define x 1))")]
    fn test_eval_src_unexpected_token() {
        let e = Evaluator::with_builtin();
        eval_src("(define x 1))", e.context());
    }
}
