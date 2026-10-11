(define (factorial n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))

(load "lib/io.rsc")
(load "lib/format.rsc")
(define n (str->num (prompt "Enter a number: ")))
(println (format "factorial({}) => {}" n (factorial n)))
