;; I/O for the example scripts. From a script in `examples/`:
;;
;;   (load "lib/io.rsc")

;; `(print expr ...)` -- write `(repr expr)` for each argument, with nothing between them.
(define (print *args)
    (apply write (map repr args)))

;; `(println expr ...)` -- `(write expr ...)` followed by a newline.
(define (println *args)
    (apply write args)
    (write "\n"))

;; `(read-num)` -- one line from stdin, parsed as a number.
(define (read-num)
    (str->num (read)))

;; `(prompt text)` -- write `text` with no newline, then `(read)`.
(define (prompt text)
    (write text)
    (read))
