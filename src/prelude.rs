use crate::eval::{eval_source, EvalContext};

const PRELUDE_SYMBOLS: [&str; 1] = [
    // Short names for `num-*` natives.
    r#"
    (define + num-add)
    (define - num-subtract)
    (define * num-multiply)
    (define / num-divide)
    (define % num-modulo)
    (define sqrt num-sqrt)
    (define exp num-exp)
    (define log num-log)
    (define expt num-expt)
    "#,
];

const PRELUDE_MACROS: [&str; 13] = [
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
    // letrec -- placeholders then set!; used by mutual recursion
    r#"
    (defmacro (letrec bindings *body)
        (if (null? bindings)
            `(begin ,@body)
            (if (atom? bindings)
                (error "letrec: bindings must be a list, got" bindings)
                (begin
                    (map (lambda (b)
                        (if (atom? b)
                            (error "letrec: each binding must be (name value), got" b)
                            (if (null? (cdr b))
                                (error "letrec: each binding must be (name value), got" b)
                                (if (null? (cddr b))
                                    (if (sym? (car b))
                                        true
                                        (error "letrec: binding name must be a symbol, got" (car b)))
                                    (error "letrec: each binding must be (name value), got" b)))))
                        bindings)
                    `(let ,(map (lambda (b) (list (car b) false)) bindings)
                        ,@(map (lambda (b) `(set! ,(car b) ,(cadr b))) bindings)
                        ,@body)))))
    "#,
    // let -- (let ((x e) ...) body) | (let (((a b) e) ...) body) destructuring |
    //        (let name ((x e) ...) body) named let. Shape checks use nested `if`
    //        to avoid expanding back into `let` during validation.
    r#"
    (defmacro (let first *rest)
        (begin
            (define (bad-binding b)
                (error "let: each binding must be (name-or-pattern value), got" b))
            (define (validate-sym-binding b)
                (if (atom? b)
                    (bad-binding b)
                    (if (null? (cdr b))
                        (bad-binding b)
                        (if (null? (cddr b))
                            (if (sym? (car b))
                                true
                                (error "let: binding name must be a symbol, got" (car b)))
                            (bad-binding b)))))
            (define (validate-binding b)
                (if (atom? b)
                    (bad-binding b)
                    (if (null? (cdr b))
                        (bad-binding b)
                        (if (null? (cddr b))
                            (begin
                                (define pat (car b))
                                (if (sym? pat)
                                    true
                                    (if (null? pat)
                                        (error "let: binding pattern must be a non-empty list of symbols, got" pat)
                                        (if (atom? pat)
                                            (error "let: binding pattern must be a non-empty list of symbols, got" pat)
                                            (begin
                                                (map (lambda (s)
                                                    (if (sym? s)
                                                        true
                                                        (error "let: pattern element must be a symbol, got" s)))
                                                    pat)
                                                true)))))
                            (bad-binding b)))))
            (define (param-name b i)
                (define pat (car b))
                (if (sym? pat) pat (str->sym (str-append "$let" (num->str i)))))
            (define (params-of bindings)
                (define (loop bindings i)
                    (if (null? bindings)
                        '()
                        (cons (param-name (car bindings) i)
                              (loop (cdr bindings) (+ i 1)))))
                (loop bindings 0))
            (define (pattern-binds pat tmp)
                (if (null? pat)
                    '()
                    (cons (list (car pat) (list 'car tmp))
                          (pattern-binds (cdr pat) (list 'cdr tmp)))))
            (define (destructure-of bindings)
                (define (loop bindings i)
                    (if (null? bindings)
                        '()
                        (begin
                            (define b (car bindings))
                            (define pat (car b))
                            (define rest (loop (cdr bindings) (+ i 1)))
                            (if (sym? pat)
                                rest
                                (append (pattern-binds pat (param-name b i)) rest)))))
                (loop bindings 0))
            (if (sym? first)
                (if (null? rest)
                    (error "let: named let requires bindings and a body")
                    (if (atom? (car rest))
                        (error "let: named let bindings must be a list, got" (car rest))
                        (begin
                            (map validate-sym-binding (car rest))
                            `((lambda ()
                                (define (,first ,@(map car (car rest))) ,@(cdr rest))
                                (,first ,@(map cadr (car rest))))))))
                (if (null? first)
                    `(begin ,@rest)
                    (if (atom? first)
                        (error "let: bindings must be a list, got" first)
                        (begin
                            (map validate-binding first)
                            (define params (params-of first))
                            (define args (map cadr first))
                            (define destruct (destructure-of first))
                            (if (null? destruct)
                                `((lambda ,params ,@rest) ,@args)
                                `((lambda ,params (let ,destruct ,@rest)) ,@args))))))))
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
    // when / unless
    r#"
    (defmacro (when condition *body)
        `(if ,condition (begin ,@body)))
    "#,
    r#"
    (defmacro (unless condition *body)
        `(if ,condition () (begin ,@body)))
    "#,
    // case -- (case key ((d1 d2) body...) (else body...))
    r#"
    (defmacro (case key *clauses)
        `(let (($case ,key))
            (cond ,@(map (lambda (clause)
                            (if (atom? clause)
                                (error "case: each clause must be a list, got" clause)
                                (if (eq? (car clause) 'else)
                                    clause
                                    (if (atom? (car clause))
                                        (error "case: clause datums must be a list, got" (car clause))
                                        `((not (eq? (member $case (quote ,(car clause))) false))
                                          ,@(cdr clause))))))
                          clauses))))
    "#,
    // define-record -- (define-record point (x y)) => make-point, point?, point-x, point-y
    r#"
    (defmacro (define-record name fields)
        (begin
            (if (sym? name)
                true
                (error "define-record: name must be a symbol, got" name))
            (if (null? fields)
                (error "define-record: fields must be a non-empty list, got" fields)
                (if (atom? fields)
                    (error "define-record: fields must be a non-empty list, got" fields)
                    true))
            (map (lambda (f)
                    (if (sym? f)
                        true
                        (error "define-record: field must be a symbol, got" f)))
                 fields)
            (define name-str (sym->str name))
            (define maker (str->sym (str-append "make-" name-str)))
            (define pred (str->sym (str-append name-str "?")))
            (define (accessors fields n)
                (if (null? fields)
                    '()
                    (cons
                        `(define (,(str->sym (str-append name-str "-" (sym->str (car fields)))) obj)
                            (list-ref obj ,n))
                        (accessors (cdr fields) (+ n 1)))))
            `(begin
                (define (,maker ,@fields) (list (quote ,name) ,@fields))
                (define (,pred obj)
                    (if (atom? obj) false (eq? (car obj) (quote ,name))))
                ,@(accessors fields 1))))
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

const PRELUDE_FUNCS: [&str; 15] = [
    // = (eq? alias)
    "(define = eq?)",
    // caar, cadr, cdar, cddr, caddr
    r#"
    (define (caar lst) (car (car lst)))
    (define (cadr lst) (car (cdr lst)))
    (define (cdar lst) (cdr (car lst)))
    (define (cddr lst) (cdr (cdr lst)))
    (define (caddr lst) (car (cddr lst)))
    "#,
    // not -- defined via `if` so that remains the sole boolean-taking form
    r#"
    (define (not x) (if x false true))
    "#,
    // list -- a procedure so it can be passed to map/apply
    r#"
    (define (list *args) args)
    "#,
    // list-ref -- 0-based; used by define-record accessors
    r#"
    (define (list-ref lst n)
        (if (= n 0) (car lst) (list-ref (cdr lst) (- n 1))))
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
    (define (num-abs x)
        (if (< x 0) (- x) x))
    (define (num-min a *rest)
        (fold (lambda (acc x) (if (< x acc) x acc)) a rest))
    (define (num-max a *rest)
        (fold (lambda (acc x) (if (< acc x) x acc)) a rest))
    (define (num-truncate x) (- x (% x 1)))
    (define (num-floor x)
        (let ((t (num-truncate x)))
            (if (or (>= x 0) (= x t)) t (- t 1))))
    (define (num-ceil x)
        (let ((t (num-truncate x)))
            (if (or (<= x 0) (= x t)) t (+ t 1))))
    (define (num-round x)
        (if (< x 0) (num-ceil (- x 0.5)) (num-floor (+ x 0.5))))
    "#,
    // string <-> list of 1-character strings (no separate char type)
    r#"
    (define (str->list s)
        (define (loop i acc)
            (if (>= i (str-length s))
                (reverse acc)
                (loop (+ i 1) (cons (str-slice s i (+ i 1)) acc))))
        (loop 0 '()))
    (define (list->str lst)
        (if (null? lst) "" (apply str-append lst)))
    (define (str-repeat s n)
        (if (or (< n 0) (not (= n (num-truncate n))))
            (error "str-repeat: count must be a non-negative integer, got" n)
            (begin
                (define (loop i acc)
                    (if (= i n) acc (loop (+ i 1) (str-append acc s))))
                (loop 0 ""))))
    "#,
];

/// Short names for prelude-defined `num-*` helpers (loaded after those helpers).
const PRELUDE_NUM_ALIASES: [&str; 1] = [r#"
    (define abs num-abs)
    (define min num-min)
    (define max num-max)
    (define truncate num-truncate)
    (define floor num-floor)
    (define ceil num-ceil)
    (define round num-round)
    "#];

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
    for src in PRELUDE_NUM_ALIASES {
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
