(define (factorial n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))

(display "Enter a number: ")
(define n (str->num (read)))
(display "factorial(" n ") => " (factorial n)) (newline)
