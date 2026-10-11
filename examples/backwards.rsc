(load "lib/writeln.rsc")

(defmacro (backwards *args)
    `(begin ,@(reverse args)))

(backwards
    (writeln "uno")
    (writeln "dos")
    (writeln "tres"))
