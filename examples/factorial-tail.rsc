(define (factorial n)
    (define (factorial-aux n acc)
        (if (= n 0)
            acc
            (factorial-aux (- n 1) (* n acc))))
    (factorial-aux n 1))

(load "lib/io.rsc")
(load "lib/format.rsc")
(define n (str->num (prompt "Enter a number: ")))
(writeln (format "factorial({}) => {}" n (factorial n)))
