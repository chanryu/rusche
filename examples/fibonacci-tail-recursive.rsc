(define (fib n)
  (define (fib-aux n a b)
    (if (= n 0)
        a
        (fib-aux (- n 1) b (+ a b))))
  (fib-aux n 0 1))

(display "Enter a number: ")
(define n (string->number (read)))
(display "fib(" n ") => " (fib n)) (newline)
