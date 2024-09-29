(define counter
    (let ((count 0))
        (lambda ()
            (set! count (+ count 1))
            count)))

(println (counter)) ; 1
(println (counter)) ; 2
(println (counter)) ; 3
