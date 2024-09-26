(define (fib-aux n a b)
  (if (= n 0)
      a
      (fib-aux (- n 1) b (+ a b))))

(define (fib n)
  (fib-aux n 0 1))


(print "Enter a number: ")
(define n (read-num))
(println "fib(" n ") => " (fib n))
