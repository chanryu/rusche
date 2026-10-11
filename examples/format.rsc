;; writeln and format. From another file in this directory:
;;
;;   (load "format.rsc")

;; `(writeln expr ...)` -- `(write expr ...)` followed by a newline.
(define (writeln *args)
    (apply write args)
    (write "\n"))

;; `(format template *args)` -- replace each `{}` with the next argument.
;; Strings are raw text; other values use their source form (`show`).
;; `{{` and `}}` are literal braces. Anything else after `{` is an error.
(define (format template *args)
    (define (text x)
        (if (str? x) x (show x)))
    (define (push s acc)
        (fold (lambda (acc ch) (cons ch acc)) acc (str->list s)))
    (define (loop cs args acc)
        (if (null? cs)
            (if (null? args)
                (list->str (reverse acc))
                (error "format: unused arguments:" (length args)))
            (if (= (car cs) "{")
                (if (null? (cdr cs))
                    (error "format: unclosed '{'")
                    (if (= (cadr cs) "{")
                        (loop (cddr cs) args (cons "{" acc))
                        (if (= (cadr cs) "}")
                            (if (null? args)
                                (error "format: not enough arguments")
                                (loop (cddr cs) (cdr args) (push (text (car args)) acc)))
                            (error "format: expected '}' after '{'"))))
                (if (= (car cs) "}")
                    (if (and (not (null? (cdr cs))) (= (cadr cs) "}"))
                        (loop (cddr cs) args (cons "}" acc))
                        (error "format: unmatched '}'"))
                    (loop (cdr cs) args (cons (car cs) acc))))))
    (if (str? template)
        (loop (str->list template) args '())
        (error "format: expected a string template, got" template)))
