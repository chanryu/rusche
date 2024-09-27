(define (factorial n)
    (define (factorial-aux n acc)
        (if (= n 0)
            acc
            (factorial-aux (- n 1) (* n acc))))
    (factorial-aux n 1))

(print "Enter a number: ")
(define n (read-num))
(println "factorial(" n ") => " (factorial n))
