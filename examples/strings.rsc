#!/usr/bin/env rusche-cli
;; String helpers: find, split, join, replace, trim, case, repeat, compare.
;;
;;   cargo run -p rusche-cli -- examples/strings.rsc

(define line "  Hello, Rusche — hello again.  ")

(print "raw:      [" line "]\n")
(print "trimmed:  [" (str-trim line) "]\n")
(print "upcase:   " (str-upcase (str-trim line)) "\n")
(print "downcase: " (str-downcase (str-trim line)) "\n")

(define words (str-split (str-trim line) " "))
(print "words:    " words "\n")
(print "joined:   " (str-join words "|") "\n")

(print "find 'che': " (str-find line "che") "\n")
(print "replace:  " (str-replace (str-trim line) "hello" "hi") "\n")
(print "repeat:   " (str-repeat "-*" 5) "\n")

(print "order:    " (< "apple" "banana") " " (= (+ "a" "b") "ab") "\n")
(print "length:   " (length line) " reversed: " (reverse "Rusche") "\n")
(print "earliest: " (min "pear" "apple" "plum") "\n")
(print "chars:    " (str->list "ab") "\n")
(print "rejoined: " (list->str '("x" "y" "z")) "\n")
