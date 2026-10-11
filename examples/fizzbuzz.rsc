#!/usr/bin/env rusche-cli
(define (fizzbuzz n)
    (define (div? n m) (= (% n m) 0))
    (cond ((div? n 15) "FizzBuzz")
          ((div? n 3) "Fizz")
          ((div? n 5) "Buzz")
          (else n)))

(load "examples/helpers.rsc")

(write "Enter a number to fizzbuzz: ")

(let ((n 1)
      (m (str->num (read)))) ; read a number from stdio and store it to `m`
    (while (<= n m)
        (writeln (fizzbuzz n))
        (set! n (+ n 1))))
