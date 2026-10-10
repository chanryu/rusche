;; Memoized Fibonacci using the dict foreign type.
;; Without the cache, the naive tree recursion is exponential; with it,
;; each n is computed once.
;;
;;   cargo run -p rusche-cli -- examples/fibonacci.rsc

(define cache (dict))

(define (fib n)
  (if (dict-has? cache n)
      (dict-get cache n)
      (let ((result (if (< n 2)
                        n
                        (+ (fib (- n 1)) (fib (- n 2))))))
        (dict-set! cache n result)
        result)))

(display "Enter a number: ")
(define n (str->num (read)))
(display "fib(" n ") => " (fib n)) (newline)
(display "cache entries: " (dict-length cache)) (newline)
