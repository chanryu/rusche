#!/usr/bin/env rusche-cli
;; Word frequencies with the dict foreign type.
;;
;;   cargo run -p rusche-cli -- examples/dict.rsc

(load "examples/helpers.rsc")

(define words '(the quick brown fox jumps over the lazy dog the fox))

(define counts (dict))

(define (tally w)
    (dict-set! counts w (+ (dict-get counts w 0) 1)))

(define (tally-all lst)
    (if (null? lst)
        ()
        (begin
            (tally (car lst))
            (tally-all (cdr lst)))))

(define (print-counts pairs)
    (if (null? pairs)
        ()
        (let ((pair (car pairs)))
            (writeln (format "{}: {}" (car pair) (cadr pair)))
            (print-counts (cdr pairs)))))

(tally-all words)
(print-counts (dict->list counts))
(writeln (format "unique: {}" (dict-length counts)))
(writeln (format "has fox? {}" (dict-has? counts 'fox)))
(writeln (format "keys: {}" (dict-keys counts)))
