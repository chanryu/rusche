;; Prelude macros. Load order matters: conditionals, binding, definitions, iteration.
;; Stored unexpanded; `cond` must precede `and` / `or` / `case`, and `let` before `let*` / `letrec`.

;; cond

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

;; and -- short-circuits, returns true or false

(defmacro (and *args)
    (cond ((null? args) true)
          ((null? (cdr args)) `(if ,(car args) true false))
          (else `(if ,(car args) (and ,@(cdr args)) false))))

;; or -- short-circuits, returns true or false

(defmacro (or *args)
    (cond ((null? args) false)
          ((null? (cdr args)) `(if ,(car args) true false))
          (else `(if ,(car args) true (or ,@(cdr args))))))

;; or-else -- returns expr unless it is false, otherwise default

(defmacro (or-else expr default)
    `((lambda (v) (if (eq? v false) ,default v)) ,expr))

;; when / unless

(defmacro (when condition *body)
    `(if ,condition (begin ,@body)))

(defmacro (unless condition *body)
    `(if ,condition () (begin ,@body)))

;; case -- (case key ((d1 d2) body...) (else body...))

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

;; let -- (let ((x e) ...) body) | (let (((a b) e) ...) body) destructuring |
;; (let name ((x e) ...) body) named let. Shape checks use nested `if`
;; to avoid expanding back into `let` during validation.

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

;; let*

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

;; letrec -- placeholders then set!; used by mutual recursion

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

;; defun

(defmacro (defun name args *body)
    `(define ,name (lambda ,args ,@body)))

;; defrecord -- (defrecord point (x y)) => make-point, point?, point-x, point-y

(defmacro (defrecord name fields)
    (begin
        (if (sym? name)
            true
            (error "defrecord: name must be a symbol, got" name))
        (if (null? fields)
            (error "defrecord: fields must be a non-empty list, got" fields)
            (if (atom? fields)
                (error "defrecord: fields must be a non-empty list, got" fields)
                true))
        (map (lambda (f)
                (if (sym? f)
                    true
                    (error "defrecord: field must be a symbol, got" f)))
             fields)
        (define name-str (sym->str name))
        (define maker (str->sym (str-append "make-" name-str)))
        (define pred (str->sym (str-append name-str "?")))
        (define (accessors fields n)
            (if (null? fields)
                '()
                (cons
                    `(define (,(str->sym (str-append name-str "-" (sym->str (car fields)))) obj)
                        (nth obj ,n))
                    (accessors (cdr fields) (+ n 1)))))
        `(begin
            (define (,maker ,@fields) (list (quote ,name) ,@fields))
            (define (,pred obj)
                (if (atom? obj) false (eq? (car obj) (quote ,name))))
            ,@(accessors fields 1))))

;; while -- the helper `loop` is scoped inside a lambda so it does not leak into the caller

(defmacro (while condition *body)
    `((lambda ()
        (define (loop)
            (if ,condition (begin ,@body (loop))))
        (loop))))
