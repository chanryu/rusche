#!/usr/bin/env rusche-cli
;; String helpers: find, split, join, replace, trim, case, repeat.
;;
;;   cargo run -p rusche-cli -- examples/strings.rsc

(define line "  Hello, Rusche — hello again.  ")

(display "raw:      [" line "]") (newline)
(display "trimmed:  [" (str-trim line) "]") (newline)
(display "upcase:   " (str-upcase (str-trim line))) (newline)
(display "downcase: " (str-downcase (str-trim line))) (newline)

(define words (str-split (str-trim line) " "))
(display "words:    " words) (newline)
(display "joined:   " (str-join words "|")) (newline)

(display "find 'che': " (str-find line "che")) (newline)
(display "replace:  " (str-replace (str-trim line) "hello" "hi")) (newline)
(display "repeat:   " (str-repeat "-*" 5)) (newline)

(display "chars:    " (str->list "ab")) (newline)
(display "rejoined: " (list->str '("x" "y" "z"))) (newline)
