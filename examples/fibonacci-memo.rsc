;; Memoized Fibonacci using the dict foreign type.
;; Without the cache, the naive tree recursion is exponential; with it,
;; each n is computed once.
;;
;;   cargo run -p rusche-cli -- examples/fibonacci-memo.rsc

(define cache (dict))

(define (fib n)
  (if (dict-has? cache n)
      (dict-get cache n)
      (let ((result (if (< n 2)
                        n
                        (+ (fib (- n 1)) (fib (- n 2))))))
        (dict-set! cache n result)
        result)))

(load "lib/io.rsc")
(load "lib/format.rsc")
(define n (str->num (prompt "Enter a number: ")))
(println (format "fib({}) => {}" n (fib n)))
(println (format "cache entries: {}" (dict-length cache)))
