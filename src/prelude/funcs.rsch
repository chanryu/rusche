;; Prelude procedures. Overloads first.
;; Bodies of early overloads use `if` only: `cond` expands to `let`, and `let` calls
;; `+` and `append` (`append` calls `reverse`). Later names (`fold`, `str->list`) are
;; resolved when a procedure runs.

;; `+` adds numbers or concatenates strings. `()` stays 0, matching `num-add`.

;; `if` only: `cond` expands to `let`, and `let` calls `+`.
(define (+ *args)
    (define (check-rest pred wanted lst)
        (if (null? lst)
            true
            (if (pred (car lst))
                (check-rest pred wanted (cdr lst))
                (error (str-append "+: expected " wanted ", got") (car lst)))))
    (if (null? args)
        0
        (if (num? (car args))
            (begin
                (check-rest num? "a number" (cdr args))
                (apply num-add args))
            (if (str? (car args))
                (begin
                    (check-rest str? "a string" (cdr args))
                    (apply str-append args))
                (error "+: expected a number or a string, got" (car args))))))

;; reverse -- tail-recursive so long lists do not hit the call depth limit.
;; Strings reverse as a list of one-character strings (`str->list` / `list->str` are below).

;; `if` only: `append` calls `reverse` while `let` is expanding, and `let` is how `cond` expands.
(define (reverse x)
    (define (loop lst acc)
        (if (null? lst) acc (loop (cdr lst) (cons (car lst) acc))))
    (if (str? x)
        (list->str (loop (str->list x) '()))
        (if (if (null? x) true (not (atom? x)))
            (loop x '())
            (error "reverse: expected a list or a string, got" x))))

;; append -- two lists, or two strings. Tail-recursive for lists.

;; `if` only: `let` calls `append` while expanding.
(define (append a b)
    (define (loop lst acc)
        (if (null? lst) acc (loop (cdr lst) (cons (car lst) acc))))
    (define (list? x) (if (null? x) true (not (atom? x))))
    (if (str? a)
        (if (str? b)
            (str-append a b)
            (error "append: expected a string, got" b))
        (if (list? a)
            (if (list? b)
                (loop (reverse a) b)
                (error "append: expected a list, got" b))
            (error "append: expected a list or a string, got" a))))

;; length -- tail-recursive for lists; strings use `str-length`

(define (length x)
    (define (loop lst n)
        (if (null? lst) n (loop (cdr lst) (+ n 1))))
    (if (str? x)
        (str-length x)
        (if (if (null? x) true (not (atom? x)))
            (loop x 0)
            (error "length: expected a list or a string, got" x))))

;; Comparisons. Helpers stay in the closure so they are not global bindings.
;; Each operator closes over that closure, so garbage collection keeps it.

(define = false)
(define < false)
(define <= false)
(define > false)
(define >= false)
((lambda (ops)
    (set! < (car ops))
    (set! <= (car (cdr ops)))
    (set! > (car (cdr (cdr ops))))
    (set! >= (car (cdr (cdr (cdr ops)))))
    (set! = (car (cdr (cdr (cdr (cdr ops)))))))
 ((lambda ()
    (define (cmp-error name wanted got)
        (error (str-append name ": expected " wanted ", got") got))
    ;; `if` only. `cond` expands through `let`, and these run on every comparison.
    (define (list? x) (if (null? x) true (not (atom? x))))
    (define (value-type x)
        (if (num? x) 'num
            (if (str? x) 'str
                (if (sym? x) 'sym
                    (if (proc? x) 'proc
                        (if (list? x) 'list
                            (if (eq? x true) 'bool
                                (if (eq? x false) 'bool
                                    'foreign))))))))
    (define (type-name t)
        (if (eq? t 'num) "a number"
            (if (eq? t 'str) "a string"
                (if (eq? t 'sym) "a symbol"
                    (if (eq? t 'proc) "a procedure"
                        (if (eq? t 'list) "a list"
                            (if (eq? t 'bool) "a boolean"
                                "a foreign object")))))))
    ;; Lexicographic `<` for lists, including a shorter list that is a prefix.
    (define (list-before? name a b)
        (if (if (null? a) (null? b) false)
            false
            (if (null? a)
                true
                (if (null? b)
                    false
                    (if (holds? name num-less (car a) (car b))
                        true
                        (if (holds? name num-less (car b) (car a))
                            false
                            (list-before? name (cdr a) (cdr b))))))))
    (define (list-rel name op a b)
        (if (eq? op num-less)
            (list-before? name a b)
            (if (eq? op num-greater)
                (list-before? name b a)
                (if (eq? op num-less-or-equal)
                    (not (list-before? name b a))
                    (not (list-before? name a b))))))
    (define (holds? name op a b)
        (if (num? a)
            (if (num? b)
                (op a b)
                (cmp-error name "a number" b))
            (if (str? a)
                (if (str? b)
                    (op (str-compare a b) 0)
                    (cmp-error name "a string" b))
                (if (list? a)
                    (if (list? b)
                        (list-rel name op a b)
                        (cmp-error name "a list" b))
                    (cmp-error name "a number, a string, or a list" a)))))
    ;; A false pair still type-checks the remaining arguments.
    (define (chain name op a b rest)
        (define (check-rest x xs)
            (if (null? xs)
                true
                (begin
                    (holds? name op x (car xs))
                    (check-rest (car xs) (cdr xs)))))
        (if (holds? name op a b)
            (if (null? rest) true (chain name op b (car rest) (cdr rest)))
            (begin
                (check-rest b rest)
                false)))
    (define (< a b *rest) (chain "<" num-less a b rest))
    (define (<= a b *rest) (chain "<=" num-less-or-equal a b rest))
    (define (> a b *rest) (chain ">" num-greater a b rest))
    (define (>= a b *rest) (chain ">=" num-greater-or-equal a b rest))
    (define (= a b *rest)
        (define (pair x y)
            (define tx (value-type x))
            (if (eq? tx (value-type y))
                (eq? x y)
                (error (str-append "=: expected " (type-name tx) ", got") y)))
        (define (loop x y rest)
            (if (pair x y)
                (if (null? rest) true (loop y (car rest) (cdr rest)))
                false))
        (loop a b rest))
    (cons < (cons <= (cons > (cons >= (cons = '()))))))))

;; `min` / `max` follow `<`. They call `fold`, defined with the list procedures below.

(define (min a *rest)
    (fold (lambda (acc x) (if (< x acc) x acc)) a rest))
(define (max a *rest)
    (fold (lambda (acc x) (if (< acc x) x acc)) a rest))

;; caar, cadr, cdar, cddr, caddr

(define (caar lst) (car (car lst)))
(define (cadr lst) (car (cdr lst)))
(define (cdar lst) (cdr (car lst)))
(define (cddr lst) (cdr (cdr lst)))
(define (caddr lst) (car (cddr lst)))

;; list -- a procedure so it can be passed to map/apply

(define (list *args) args)

;; nth -- 0-based; used by defrecord accessors

(define (nth lst n)
    (if (= n 0) (car lst) (nth (cdr lst) (- n 1))))

;; map -- tail-recursive

(define (map proc lst)
    (define (loop lst acc)
        (if (null? lst)
            (reverse acc)
            (loop (cdr lst) (cons (proc (car lst)) acc))))
    (loop lst '()))

;; filter -- tail-recursive

(define (filter pred lst)
    (define (loop lst acc)
        (if (null? lst)
            (reverse acc)
            (loop (cdr lst)
                  (if (pred (car lst))
                      (cons (car lst) acc)
                      acc))))
    (loop lst '()))

;; fold -- left fold, tail-recursive

(define (fold proc init lst)
    (if (null? lst)
        init
        (fold proc (proc init (car lst)) (cdr lst))))

;; member

(define (member x lst)
    (cond
        ((null? lst) false)
        ((eq? (car lst) x) lst)
        (true (member x (cdr lst)))))

;; assoc

(define (assoc key lst)
    (cond
        ((null? lst) false)                    ; If the list is empty, return false
        ((eq? (car (car lst)) key) (car lst))  ; If the car of the first element matches the key, return the pair
        (true (assoc key (cdr lst)))))         ; Otherwise, recursively search the rest of the list

;; not -- defined via `if` so that remains the sole boolean-taking form

(define (not x) (if x false true))

;; `num-min` / `num-max` stay numeric. `min` / `max` are the overloads above.

(define (num-abs x)
    (if (< x 0) (- x) x))
(define (num-min a *rest)
    (fold (lambda (acc x) (if (num-less x acc) x acc)) a rest))
(define (num-max a *rest)
    (fold (lambda (acc x) (if (num-less acc x) x acc)) a rest))
(define (num-truncate x) (- x (% x 1)))
(define (num-floor x)
    (let ((t (num-truncate x)))
        (if (or (>= x 0) (= x t)) t (- t 1))))
(define (num-ceil x)
    (let ((t (num-truncate x)))
        (if (or (<= x 0) (= x t)) t (+ t 1))))
(define (num-round x)
    (if (< x 0) (num-ceil (- x 0.5)) (num-floor (+ x 0.5))))

;; string <-> list of 1-character strings (no separate char type)

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
