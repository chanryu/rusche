use crate::eval::{eval_source, EvalContext};

const PRELUDE_SYMBOLS: [&str; 1] = [
    // numeric operation aliases
    r#"
    (define + num-add)
    (define - num-subtract)
    (define * num-multiply)
    (define / num-divide)
    (define % num-modulo)
    "#,
];

const PRELUDE_MACROS: [&str; 8] = [
    // cond
    r#"
    (defmacro (cond *clauses)
        (if (null? clauses)
            false                                       ; No more clauses, return false by default
            (let ((clause (car clauses)))
                (if (atom? clause)
                    (error "cond: each clause must be a list, got" clause)
                    (if (eq? (car clause) 'else)        ; If the first clause is 'else'
                        `(begin ,@(cdr clause))         ; Expand to the else expression(s)
                        `(if ,(car clause)              ; Otherwise, expand to an if expression
                            (begin ,@(cdr clause))      ; If condition is true, evaluate the body
                            (cond ,@(cdr clauses))))))))
    "#,
    // defun
    r#"
    (defmacro (defun name args *body)
        `(define ,name (lambda ,args ,@body)))
    "#,
    // let -- shape checks use nested `if` (not `or`/`cond`) to avoid expanding back into `let`.
    // `()` is an atom in Rusche, so check `null?` before `atom?` when empty bindings are allowed.
    r#"
    (defmacro (let bindings *body)
        (if (null? bindings)
            `(begin ,@body)
            (if (atom? bindings)
                (error "let: bindings must be a list, got" bindings)
                (begin
                    (map (lambda (b)
                        (if (atom? b)
                            (error "let: each binding must be (name value), got" b)
                            (if (null? (cdr b))
                                (error "let: each binding must be (name value), got" b)
                                (if (null? (cddr b))
                                    (if (sym? (car b))
                                        true
                                        (error "let: binding name must be a symbol, got" (car b)))
                                    (error "let: each binding must be (name value), got" b)))))
                        bindings)
                    `((lambda ,(map car bindings)
                         ,@body)
                      ,@(map cadr bindings))))))
    "#,
    // let*
    r#"
    (defmacro (let* bindings *body)
        (if (null? bindings)
            `(begin ,@body)
            (if (atom? bindings)
                (error "let*: bindings must be a list, got" bindings)
                (let ((binding (car bindings)))
                    (if (atom? binding)
                        (error "let*: each binding must be (name value), got" binding)
                        (if (null? (cdr binding))
                            (error "let*: each binding must be (name value), got" binding)
                            (if (null? (cddr binding))
                                `(let ((,(car binding) ,(cadr binding)))
                                    (let* ,(cdr bindings) ,@body))
                                (error "let*: each binding must be (name value), got" binding))))))))
    "#,
    // while -- the helper `loop` is scoped inside a lambda so it does not leak into the caller
    r#"
    (defmacro (while condition *body)
        `((lambda ()
            (define (loop)
                (if ,condition (begin ,@body (loop))))
            (loop))))
    "#,
    // and -- short-circuits, returns true or false
    r#"
    (defmacro (and *args)
        (cond ((null? args) true)
              ((null? (cdr args)) `(if ,(car args) true false))
              (else `(if ,(car args) (and ,@(cdr args)) false))))
    "#,
    // or -- short-circuits, returns true or false
    r#"
    (defmacro (or *args)
        (cond ((null? args) false)
              ((null? (cdr args)) `(if ,(car args) true false))
              (else `(if ,(car args) true (or ,@(cdr args))))))
    "#,
    // or-else -- returns expr unless it is false, otherwise default
    r#"
    (defmacro (or-else expr default)
        `((lambda (v) (if (eq? v false) ,default v)) ,expr))
    "#,
];

const PRELUDE_FUNCS: [&str; 13] = [
    // = (eq? alias)
    "(define = eq?)",
    // caar, cadr, cdar, cddr
    r#"
    (define (caar lst) (car (car lst)))
    (define (cadr lst) (car (cdr lst)))
    (define (cdar lst) (cdr (car lst)))
    (define (cddr lst) (cdr (cdr lst)))
    "#,
    // not
    r#"
    (define (not x) (if x false true))
    "#,
    // list -- a procedure so it can be passed to map/apply
    r#"
    (define (list *args) args)
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
    (define (map proc lst)
        (define (loop lst acc)
            (if (null? lst)
                (reverse acc)
                (loop (cdr lst) (cons (proc (car lst)) acc))))
        (loop lst '()))
    "#,
    // append -- tail-recursive
    r#"
    (define (append lst1 lst2)
        (define (loop lst acc)
            (if (null? lst) acc (loop (cdr lst) (cons (car lst) acc))))
        (loop (reverse lst1) lst2))
    "#,
    // length -- tail-recursive
    r#"
    (define (length lst)
        (define (loop lst n)
            (if (null? lst) n (loop (cdr lst) (+ n 1))))
        (loop lst 0))
    "#,
    // filter -- tail-recursive
    r#"
    (define (filter pred lst)
        (define (loop lst acc)
            (if (null? lst)
                (reverse acc)
                (loop (cdr lst)
                      (if (pred (car lst))
                          (cons (car lst) acc)
                          acc))))
        (loop lst '()))
    "#,
    // fold -- left fold, tail-recursive
    r#"
    (define (fold proc init lst)
        (if (null? lst)
            init
            (fold proc (proc init (car lst)) (cdr lst))))
    "#,
    // member
    r#"
    (define (member x lst)
        (cond
            ((null? lst) false)
            ((eq? (car lst) x) lst)
            (true (member x (cdr lst)))))
    "#,
    // assoc
    r#"
    (define (assoc key lst)
        (cond
            ((null? lst) false)                    ; If the list is empty, return false
            ((eq? (car (car lst)) key) (car lst))  ; If the car of the first element matches the key, return the pair
            (true (assoc key (cdr lst)))))         ; Otherwise, recursively search the rest of the list
    "#,
    // numeric operations
    r#"
    (define (< a b *rest)
        (if (num-less a b)
            (if (null? rest) true (apply < (cons b rest)))
            false))
    (define (<= a b *rest)
        (if (or (num-less a b) (= a b))
            (if (null? rest) true (apply <= (cons b rest)))
            false))
    (define (> a b *rest)
        (if (num-less b a)
            (if (null? rest) true (apply > (cons b rest)))
            false))
    (define (>= a b *rest)
        (if (or (num-less b a) (= a b))
            (if (null? rest) true (apply >= (cons b rest)))
            false))
    (define (abs x)
        (if (< x 0) (- x) x))
    (define (min a *rest)
        (fold (lambda (acc x) (if (< x acc) x acc)) a rest))
    (define (max a *rest)
        (fold (lambda (acc x) (if (< acc x) x acc)) a rest))
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
    // Prelude source locations are meaningless to users; strip them so errors
    // raised inside prelude code are attributed to the user's call site instead.
    if let Err(error) = eval_source(src, context, true) {
        use crate::error::Error;
        match error {
            Error::Lex(_) => panic!("Prelude tokniization failed: {}", src),
            Error::Parse(crate::parser::ParseError::IncompleteExpr(_)) => {
                panic!("Prelude parse failure - incomplete expression: {}", src);
            }
            Error::Parse(crate::parser::ParseError::UnexpectedToken(token)) => {
                panic!(
                    "Prelude parse failure - unexpected token \"{}\": {}",
                    token, src
                );
            }
            Error::Eval(_) => panic!("Prelude evaluation failed: {}", src),
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
