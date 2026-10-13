; Mutable UI state shared across panels.

(define script (buffer initial-script))
(define log-buf (buffer ""))
(define repl-input (buffer ""))
(define tab 'script)
(define show-help false)
(define speed 2)
(define repl-history '())
(define history-pos false)
(define focus-repl false)
(define cursor-end false)

(define speeds (list "Crawl" "Walk" "Trot" "Run" "Dash"))

(define (append-script text)
  (let ((cur (buffer-text script)))
    (if (if (= cur "") false (not (str-ends-with? cur "\n")))
        (buffer-set! script (str-append cur "\n" text "\n"))
        (buffer-set! script (str-append cur text "\n")))))
