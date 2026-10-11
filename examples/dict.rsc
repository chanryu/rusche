#!/usr/bin/env rusche-cli
;; Word frequencies with the dict foreign type.
;;
;;   cargo run -p rusche-cli -- examples/dict.rsc

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
            (print (car pair) ": " (cadr pair) "\n")
            (print-counts (cdr pairs)))))

(tally-all words)
(print-counts (dict->list counts))
(print "unique: " (dict-length counts) "\n")
(print "has fox? " (dict-has? counts 'fox) "\n")
(print "keys: " (dict-keys counts) "\n")
