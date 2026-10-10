#!/usr/bin/env rusche-cli
;; Word frequencies with the dict foreign type.
;;
;;   cargo run -p rusche-cli -- examples/dict.rsc

(define words '(the quick brown fox jumps over the lazy dog the fox))

(define counts (dict-make))

(defun tally (w)
    (dict-set! counts w (+ (dict-get counts w 0) 1)))

(defun tally-all (lst)
    (if (null? lst)
        ()
        (begin
            (tally (car lst))
            (tally-all (cdr lst)))))

(defun print-counts (pairs)
    (if (null? pairs)
        ()
        (let ((pair (car pairs)))
            (display (car pair) ": " (cadr pair))
            (newline)
            (print-counts (cdr pairs)))))

(tally-all words)
(print-counts (dict->list counts))
(display "unique: " (dict-length counts)) (newline)
(display "has fox? " (dict-has? counts 'fox)) (newline)
(display "keys: " (dict-keys counts)) (newline)
