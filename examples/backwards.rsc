(defmacro (backwards *args)
    `(begin ,@(reverse args)))

(backwards
    (display "uno\n")
    (display "dos\n")
    (display "tres\n"))
