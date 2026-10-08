(define (fib n)
  (if (< n 2)
      n
      (+ (fib (- n 1)) (fib (- n 2)))))

(display "Enter a number: ")
(define n (string->number (read)))
(display "fib(" n ") => " (fib n)) (newline)
