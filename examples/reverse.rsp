(define (reverse lst)
    (if (null? lst) lst
        (append (reverse (cdr lst)) (list (car lst)))))

(define (println x)
    (display x)
    (display "\n"))

(println (reverse '(a b c d))) ; --> (d c b a)
