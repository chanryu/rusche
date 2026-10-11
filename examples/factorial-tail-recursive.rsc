(define (factorial n)
    (define (factorial-aux n acc)
        (if (= n 0)
            acc
            (factorial-aux (- n 1) (* n acc))))
    (factorial-aux n 1))

(load "examples/helpers.rsc")

(write "Enter a number: ")
(define n (str->num (read)))
(writeln (format "factorial({}) => {}" n (factorial n)))
