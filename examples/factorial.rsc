(define (factorial n)
    (if (= n 0)
        1
        (* n (factorial (- n 1)))))

(print "Enter a number: ")
(define n (num-parse (read)))
(println "factorial(" n ") => " (factorial n))
