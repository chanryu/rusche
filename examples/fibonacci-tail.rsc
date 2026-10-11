(define (fib n)
  (define (fib-aux n a b)
    (if (= n 0)
        a
        (fib-aux (- n 1) b (+ a b))))
  (fib-aux n 0 1))

(load "lib/io.rsc")
(load "lib/format.rsc")
(define n (str->num (prompt "Enter a number: ")))
(writeln (format "fib({}) => {}" n (fib n)))
