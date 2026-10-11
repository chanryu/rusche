(define (factorial n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))

(load "lib/writeln.rsc")
(load "lib/format.rsc")

(write "Enter a number: ")
(define n (str->num (read)))
(writeln (format "factorial({}) => {}" n (factorial n)))
