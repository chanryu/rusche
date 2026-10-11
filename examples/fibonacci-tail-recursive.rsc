(define (fib n)
  (define (fib-aux n a b)
    (if (= n 0)
        a
        (fib-aux (- n 1) b (+ a b))))
  (fib-aux n 0 1))

(load "helpers.rsc")

(write "Enter a number: ")
(define n (str->num (read)))
(writeln (format "fib({}) => {}" n (fib n)))
