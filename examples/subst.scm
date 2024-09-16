(define (subst new old lst)
    (cond
        ((null? lst) '())                                  ; If the list is empty, return an empty list
        ((eq? (car lst) old)                               ; If the first element matches 'old'
        (cons new (subst new old (cdr lst))))              ; Replace it with 'new' and recurse on the rest
        (#t (cons (car lst) (subst new old (cdr lst))))))  ; Otherwise, keep the first element and recurse


(define (println x)
    (display x)
    (display "\n"))

(println
    (subst 'a 'b '(a b c b))) ; --> "(a a c a)"
