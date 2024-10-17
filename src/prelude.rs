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

const PRELUDE_MACROS: [&str; 6] = [
    // begin
    r#"
    (defmacro begin (*exprs)
        `(let () ,@exprs))
    "#,
    // cond
    r#"
    (defmacro (cond *clauses)
        (if (null? clauses)
            #f                                          ; No more clauses, return #f by default
            (let ((clause (car clauses)))
                (if (eq? (car clause) 'else)            ; If the first clause is 'else'
                    `(,@(cdr clause))                   ; Expand to the else expression(s)
                    `(if ,(car clause)                  ; Otherwise, expand to an if expression
                        (begin ,@(cdr clause))          ; If condition is true, evaluate the body
                        (cond ,@(cdr clauses)))))))     ; Else, recursively process remaining clauses
    "#,
    // defun
    r#"
    (defmacro defun (name args *body)
        `(define ,name (lambda ,args ,@body)))
    "#,
    // let
    r#"
    (defmacro let (bindings *body)
        `((lambda ,(map car bindings) ; Get the list of variable names
             ,@body)                  ; The body of the let becomes the lambda's body
          ,@(map cdar bindings)))     ; Apply the values to the lambda
    "#,
    // list
    r#"
    (defmacro list (*args)
        (if (null? args)
            '()
            `(cons ,(car args) (list ,@(cdr args)))))
    "#,
    // while
    r#"
    (defmacro while (condition *body)
        `(define (loop)
            (if ,condition (begin ,@body (loop))))
        (loop))
    "#,
];

const PRELUDE_FUNCS: [&str; 11] = [
    // caar, cadr, cdar, cdar
    r#"
    (define (caar lst) (car (car lst)))
    (define (cadr lst) (cdr (car lst)))
    (define (cdar lst) (car (cdr lst)))
    (define (cddr lst) (cdr (cdr lst)))
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
    // map
    r#"
    (define (map fn lst)
        (if (null? lst)
            '()                          ; Base case: empty list
            (cons (fn (car lst))         ; Apply function to the first element
                  (map fn (cdr lst)))))  ; Recursive call on the rest of the list
    "#,
    // append
    r#"
    (define (append lst1 lst2)
        (if (null? lst1) lst2                             ; If lst1 is empty, return lst2
            (cons (car lst1) (append (cdr lst1) lst2))))  ; Otherwise, prepend the first element of lst1 and recurse
    "#,
    // apply
    r#"
    (define (apply f args)
        (eval (cons f args)))
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
    let tokens = tokenize(src).expect(&format!("Prelude tokniization failed: {}", src));

    let mut parser = Parser::with_tokens(tokens);

    loop {
        match parser.parse() {
            Ok(None) => {
                break; // we're done!
            }
            Ok(Some(expr)) => {
                let _ = eval(&expr, context).expect(&format!("Prelude evaluation failed: {}", src));
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
