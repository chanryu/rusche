; Shared list / string helpers for the turtle UI.

(define (cadddr lst) (car (cdr (cddr lst))))
(define (cddddr lst) (cdr (cdr (cddr lst))))

(define (for-each proc lst)
  (if (null? lst)
      '()
      (begin
        (proc (car lst))
        (for-each proc (cdr lst)))))

(define (str-ends-with? s suffix)
  (let ((n (str-length s))
        (m (str-length suffix)))
    (if (< n m)
        false
        (= (str-slice s (- n m) n) suffix))))
