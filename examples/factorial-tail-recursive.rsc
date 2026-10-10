(defun factorial (n)
    (defun factorial-aux (n acc)
        (if (= n 0)
            acc
            (factorial-aux (- n 1) (* n acc))))
    (factorial-aux n 1))

(display "Enter a number: ")
(define n (str->num (read)))
(display "factorial(" n ") => " (factorial n)) (newline)
