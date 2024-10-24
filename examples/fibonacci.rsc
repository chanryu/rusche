(define (fib n)
  (if (< n 2)
      n
      (+ (fib (- n 1)) (fib (- n 2)))))

(print "Enter a number: ")
(define n (num-parse (read)))
(println "fib(" n ") => " (fib n))
