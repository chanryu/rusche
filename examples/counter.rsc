(define counter
    (let ((count 0))
        (lambda ()
            (set! count (+ count 1))
            count)))

(print (counter) "\n") ; 1
(print (counter) "\n") ; 2
(print (counter) "\n") ; 3
