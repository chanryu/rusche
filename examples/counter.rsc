(define counter
    (let ((count 0))
        (lambda ()
            (set! count (+ count 1))
            count)))

(display (counter)) (newline) ; 1
(display (counter)) (newline) ; 2
(display (counter)) (newline) ; 3
