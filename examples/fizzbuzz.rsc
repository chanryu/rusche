(defun fizzbuzz (n)
    (defun div? (n m) (= (% n m) 0))
    (cond ((div? n 15) "FizzBuzz")
          ((div? n 3) "Fizz")
          ((div? n 5) "Buzz")
          (#t n)))

(print "Enter a number to fizzbuzz: ")

(let ((n 1)
      (m (num-parse (read)))) ; read a number from stdio and store it to `m`
    (while (<= n m)
        (println (fizzbuzz n))
        (set! n (+ n 1))))
