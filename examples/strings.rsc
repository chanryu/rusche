#!/usr/bin/env rusche-cli
;; String helpers: find, split, join, replace, trim, case, repeat, compare.
;;
;;   cargo run -p rusche-cli -- examples/strings.rsc

(load "lib/io.rsc")
(load "lib/format.rsc")
(define line "  Hello, Rusche — hello again.  ")

(println (format "raw:      [{}]" line))
(println (format "trimmed:  [{}]" (str-trim line)))
(println (format "upcase:   {}" (str-upcase (str-trim line))))
(println (format "downcase: {}" (str-downcase (str-trim line))))

(define words (str-split (str-trim line) " "))
(println (format "words:    {}" words))
(println (format "joined:   {}" (str-join words "|")))

(println (format "find 'che': {}" (str-find line "che")))
(println (format "replace:  {}" (str-replace (str-trim line) "hello" "hi")))
(println (format "repeat:   {}" (str-repeat "-*" 5)))

(println (format "order:    {} {}" (< "apple" "banana") (= (+ "a" "b") "ab")))
(println (format "length:   {} reversed: {}" (length line) (reverse "Rusche")))
(println (format "earliest: {}" (min "pear" "apple" "plum")))
(println (format "chars:    {}" (str->list "ab")))
(println (format "rejoined: {}" (list->str '("x" "y" "z"))))
