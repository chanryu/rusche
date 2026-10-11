(load "format.rsc")

(define counter
    (let ((count 0))
        (lambda ()
            (set! count (+ count 1))
            count)))

(writeln (counter)) ; 1
(writeln (counter)) ; 2
(writeln (counter)) ; 3
