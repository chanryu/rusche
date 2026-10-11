(defmacro (backwards *args)
    `(begin ,@(reverse args)))

(backwards
    (print "uno\n")
    (print "dos\n")
    (print "tres\n"))
