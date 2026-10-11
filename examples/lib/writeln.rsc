;; `(writeln expr ...)` -- `(write expr ...)` followed by a newline.
;; From a script in `examples/`:
;;
;;   (load "lib/writeln.rsc")

(define (writeln *args)
    (apply write args)
    (write "\n"))
