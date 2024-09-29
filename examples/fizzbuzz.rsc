(define (fizzbuzz n)
    (define (mod0 n m) (= (% n m) 0))
    (cond ((mod0 n 15) "FizzBuzz")
          ((mod0 n 3) "Fizz")
          ((mod0 n 5) "Buzz")
          (#t n)))

(print "Enter a number: ")

(let ((n 1)
      (m (read-num)))
    (while (<= n m)
        (println (fizzbuzz n))
        (set! n (+ n 1))))
