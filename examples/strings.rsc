#!/usr/bin/env rusche-cli
;; String helpers: find, split, join, replace, trim, case, repeat, compare.
;;
;;   cargo run -p rusche-cli -- examples/strings.rsc

(load "lib/io.rsc")
(load "lib/format.rsc")
(define line "  Hello, Rusche — hello again.  ")

(writeln (format "raw:      [{}]" line))
(writeln (format "trimmed:  [{}]" (str-trim line)))
(writeln (format "upcase:   {}" (str-upcase (str-trim line))))
(writeln (format "downcase: {}" (str-downcase (str-trim line))))

(define words (str-split (str-trim line) " "))
(writeln (format "words:    {}" words))
(writeln (format "joined:   {}" (str-join words "|")))

(writeln (format "find 'che': {}" (str-find line "che")))
(writeln (format "replace:  {}" (str-replace (str-trim line) "hello" "hi")))
(writeln (format "repeat:   {}" (str-repeat "-*" 5)))

(writeln (format "order:    {} {}" (< "apple" "banana") (= (+ "a" "b") "ab")))
(writeln (format "length:   {} reversed: {}" (length line) (reverse "Rusche")))
(writeln (format "earliest: {}" (min "pear" "apple" "plum")))
(writeln (format "chars:    {}" (str->list "ab")))
(writeln (format "rejoined: {}" (list->str '("x" "y" "z"))))
